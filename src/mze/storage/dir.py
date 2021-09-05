#!/usr/bin/env python3
#
# Copyright 2021 Maksym Medvied
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

"""C.sts.dir: a directory is served directly over HTTP

This storage server has a directory where all the data is located and it
provides HTTP API to serve files from that directory.

Storage format: just plain files in the dir with UUID as filename.

TODO add error handling
TODO make the implementation fully asynchronous with async/await
"""

import os
import sys
import uuid
import shutil
import logging
import pathlib
import aiofiles

from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path
from aiohttp import web
from typing import Any, Callable, Awaitable, Optional

from mze.api.storage import BlobId, BlobInfo, BlobData, Storage


logger = logging.getLogger(__name__)


class StorageDir(Storage):
    """
    API

    - init()/fini()
    - create()/destroy()
    - get()
    - put()
    - head()
    - catalog()
    - delete()
    """
    path: pathlib.Path

    def init(self, cfg: dict[str, Any]) -> None:
        """
        Parameters in cfg:
        path: Union[str, pathlib.Path]
        """
        logger.debug(f'{cfg=}')
        self.path = pathlib.Path(cfg['path'])

    def fini(self) -> None:
        logger.debug(f'{self.path=}')

    def create(self, cfg: dict[str, Any]) -> None:
        self.init(cfg)
        self.path.mkdir(parents=True)

    def destroy(self) -> None:
        self.path.rmdir()

    def fsck(self) -> None:
        pass

    def get(self, bids: Sequence[BlobId]) -> list[Optional[BlobData]]:
        return [self._get_one(bid) for bid in bids]

    def put(self, blobs: Sequence[tuple[Optional[BlobId], BlobData]]) -> \
            list[tuple[BlobId, BlobInfo]]:
        return [self._put_one(bid, data) for bid, data in blobs]

    def head(self, bids: Sequence[BlobId]) -> list[Optional[BlobInfo]]:
        return [self._head_one(bid) for bid in bids]

    def catalog(self) -> list[tuple[BlobId, BlobInfo]]:
        return [(BlobId(bid=uuid.UUID(f.name)),
                 BlobInfo(size=f.stat().st_size))
                for f in self.path.iterdir()]

    def delete(self, bids: Sequence[BlobId]) -> list[Optional[BlobInfo]]:
        return [self._delete_one(bid) for bid in bids]

    def _get_one(self, bid: BlobId) -> Optional[BlobData]:
        file_path = self.path / str(bid.bid)
        if not file_path.exists() or not file_path.is_file():
            return None
        return BlobData(data=None, path=file_path)

    def _put_one(self, bid: Optional[BlobId], data: BlobData) -> \
            tuple[BlobId, BlobInfo]:
        if bid is None:
            bid = BlobId(bid=uuid.uuid4())
        info = self._head_one(bid)
        assert info is None, info
        assert (data.data is not None) + (data.path is not None) == 1, \
            (bid, data)
        file_path = self.path / str(bid.bid)
        if data.path is not None:
            shutil.copyfile(data.path, file_path)
        elif data.data is not None:
            with open(file_path, 'wb') as f:
                f.write(data.data)
        info = self._head_one(bid)
        assert info is not None, (bid, info, data)
        return (bid, info)

    def _head_one(self, bid: BlobId) -> Optional[BlobInfo]:
        file_path = self.path / str(bid.bid)
        if not file_path.exists():
            return None
        stat_result = file_path.stat()
        return BlobInfo(size=stat_result.st_size)

    def _delete_one(self, bid: BlobId) -> Optional[BlobInfo]:
        file_path = self.path / str(bid.bid)
        if not file_path.exists():
            return None
        info = self._head_one(bid)
        file_path.unlink()
        return info


class StorageDirCLI:
    pass


@dataclass
class RequestContext:
    op: str
    instance_uuid: uuid.UUID
    params: dict[str, str]
    storage_dir: Path
    request: web.Request


def check_request_context(rctx: RequestContext) -> None:
    for k, v in rctx.params.items():
        if k not in ['instance', 'record', 'version']:
            raise web.HTTPBadRequest(
                reason=f'{k} is not one of ["instance", "record", "version"]')
        if (k, v) in [('instance', 'any'), ('instance', 'all'),
                      ('version', 'all')]:
            continue
        try:
            if str(uuid.UUID(v)) != v:
                raise web.HTTPBadRequest(reason='invalid UUID format: '
                                         f'uuid.UUID("{v}") != "{v}" for {k}')
        except ValueError as e:
            raise web.HTTPBadRequest(reason=f'uuid.UUID("{v}") failed '
                                     f'for "{k}": {repr(e)}.')
        if k == 'instance' and v != str(rctx.instance_uuid):
            raise web.HTTPBadRequest(
                reason=f'The request is for instance {v}, '
                f'but the current instance is {rctx.instance_uuid}.')


def all_versions(record_dir: Path) -> dict[int, str]:
    assert record_dir.exists(), record_dir
    all_files = [f.name for f in record_dir.iterdir()]
    logger.debug(f'{all_files=}')
    result: dict[int, str] = {}
    for v, u in [(int(s[:s.index('-')]), s[s.index('-') + 1:])
                 for s in all_files]:
        assert v not in result, (v, u, result)
        result[v] = u
    logger.debug(f'{result=}')
    assert not (set(result.keys()) ^ set(range(len(result)))), result
    return result


async def handle_list(rctx: RequestContext) -> web.StreamResponse:
    """
    instance is already one of ['all', 'any', instance_uuid] or not specified.
    What could be requested:

    - record and version are UUIDs: return record and version if they are
      present, return empty dict otherwise
    - record is UUID, version is absent or version is 'all': return the sorted
      list of all versions for this record
    - record is absent, version is UUID: invalid request
    - record & version are absent: return the list of all records without
      versions
    - record is absent, version is 'all': return the list of all records and
      their versions
    """

    result: dict[str, list[str]] = {}
    if 'record' in rctx.params:
        versions = all_versions(rctx.storage_dir / rctx.params['record'])
        if len(versions) > 0:
            if 'version' not in rctx.params:
                result = {rctx.params['record']:
                          [versions[max(versions.keys())]]}
            elif rctx.params['version'] == 'all':
                result = {rctx.params['record']:
                          [v for k, v in sorted(versions.items())]}
            else:
                if rctx.params['version'] in versions.values():
                    result = {rctx.params['record']: [rctx.params['version']]}
    elif 'version' not in rctx.params or rctx.params['version'] == 'all':
        d = [r for r in rctx.storage_dir.iterdir()]
        logger.debug(f'{d=}')
        for record_dir in rctx.storage_dir.iterdir():
            if 'version' in rctx.params:
                result[record_dir.name] = \
                    [v for k, v in sorted(all_versions(record_dir).items())]
            else:
                result[record_dir.name] = []
    else:
        raise web.HTTPBadRequest(
            reason='list operation requires record or record and version or '
            'record and version="all", '
            'but got no record and version != "all".')
    logger.debug(f'{result=}')
    return web.json_response({str(rctx.instance_uuid): result})


async def handle_put(rctx: RequestContext) -> web.StreamResponse:
    # TODO check instance
    if 'version' in rctx.params:
        raise web.HTTPBadRequest(reason='version it not yet supported for put')
    if 'record' in rctx.params:
        record_uuid = uuid.UUID(rctx.params['record'])
    else:
        record_uuid = uuid.uuid4()
    record_dir = rctx.storage_dir / str(record_uuid)
    # XXX might be an issue with concurrent put/delete
    if not record_dir.exists():
        record_dir.mkdir(exist_ok=True)
    version_uuid = uuid.uuid4()
    # XXX handle concurrent puts for the same record
    versions = all_versions(record_dir)
    if len(versions) == 0:
        new_version_num = 0
    else:
        new_version_num = max(versions.keys()) + 1
    new_version = record_dir / f'{new_version_num}-{version_uuid}'
    async with aiofiles.open(new_version, 'xb') as f:
        # TODO handle partial upload
        async for data in rctx.request.content.iter_any():
            await f.write(data)
    return web.json_response({str(rctx.instance_uuid):
                              {str(record_uuid): [str(version_uuid)]}})


async def handle_get(rctx: RequestContext) -> web.StreamResponse:
    # TODO check instance
    if 'record' not in rctx.params:
        raise web.HTTPBadRequest(
            reason='record UUID is required for get requests')
    else:
        record_dir = rctx.storage_dir / rctx.params['record']
    if not record_dir.exists():
        raise web.HTTPNotFound(
            reason=f'record {rctx.params["record"]} doesn\'t exist.')
    av = all_versions(record_dir)
    if 'version' in rctx.params:
        version_uuid = rctx.params["version"]
        if version_uuid not in av.values():
            raise web.HTTPNotFound(
                reason=f'record {rctx.params["record"]} '
                f'doesn\'t have version {version_uuid}.')
        av_reverse = {v: k for k, v in av.items()}
        assert len(av) == len(av_reverse), (av, av_reverse)
        version_number = av_reverse[version_uuid]
    else:
        version_number = max(av.keys())
        version_uuid = av[version_number]
    return web.FileResponse(
        path=record_dir / f'{version_number}-{version_uuid}')


async def handle_not_implemented_yet(rctx: RequestContext) -> \
        web.StreamResponse:
    return web.Response(text='Not implemented yet.')


@dataclass
class OpHandler:
    method: Any
    handler: Callable[[RequestContext], Awaitable[web.StreamResponse]]


OPERATIONS: dict[str, OpHandler] = {
    'get': OpHandler(web.get, handle_get),
    'put': OpHandler(web.put, handle_put),
    'head': OpHandler(web.head, handle_not_implemented_yet),
    'list': OpHandler(web.get, handle_list),
    'delete': OpHandler(web.delete, handle_not_implemented_yet),
}


async def handler(request: web.Request) -> web.StreamResponse:
    assert request.path.startswith(request.app['web_location']), \
        f'{request.path=} {request.app["web_location"]=}'
    op = request.path[len(request.app['web_location']):]
    if op[0] == '/':
        op = op[1:]
    assert op in OPERATIONS.keys(), f'{op=} {OPERATIONS.keys()=}'
    instance_uuid = request.app['instance_uuid']
    params = {k: v for k, v in request.query.items()}
    print(f'{op=} {request.path=} {params=}', flush=True)
    rctx = RequestContext(op, instance_uuid, params,
                          request.app['storage_dir'], request)
    check_request_context(rctx)
    return await OPERATIONS[op].handler(rctx)


def main() -> None:
    logging.basicConfig(
        format='%(asctime)s %(thread)d %(levelname)s '
        '%(pathname)s:%(lineno)d:%(funcName)s %(message)s',
        level=logging.DEBUG)

    app = web.Application()

    storage_server_dir = web.Application()
    for op, op_handler in OPERATIONS.items():
        storage_server_dir.add_routes([op_handler.method(f'/{op}', handler)])
    app.add_subapp(os.environ['MZE_WEB_LOCATION'], storage_server_dir)
    if 'MZE_INSTANCE_UUID' not in os.environ:
        print('Please set MZE_INSTANCE_UUID. Can\'t start without it.',
              file=sys.stderr)
        return
    instance_uuid_s = os.environ['MZE_INSTANCE_UUID']
    try:
        instance_uuid = uuid.UUID(instance_uuid_s)
        if str(instance_uuid) != instance_uuid_s:
            print('invalid UUID format for MZE_INSTANCE_UUID: '
                  f'uuid.UUID("{instance_uuid_s}") = {instance_uuid} which '
                  f'is not equal to the UUID specified: {instance_uuid_s}.')
            return
    except ValueError as e:
        print(f'uuid.UUID(os.environ["MZE_INSTANCE_UUID"]) failed: {repr(e)}.')
        return
    storage_dir = Path(os.environ['MZE_STORAGE_DIR'])
    if not storage_dir.exists():
        storage_dir.mkdir(parents=True)
    storage_server_dir['web_location'] = os.environ['MZE_WEB_LOCATION']
    storage_server_dir['instance_uuid'] = instance_uuid
    storage_server_dir['storage_dir'] = storage_dir
    web.run_app(app, port=80)


if __name__ == '__main__':
    main()
