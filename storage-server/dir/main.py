#!/usr/bin/env python3

"""C.ss.dir: a directory is served directly over HTTP

This storage server has a directory where all the data is located and it
provides HTTP API to serve files from that directory.

Storage format:

record_uuid/version_number-version-uuid

Version numbers are defined as monotonically increasing integer sequence which
starts from 0.
"""

import os
import sys
import uuid

from dataclasses import dataclass
from pathlib import Path
from aiohttp import web
from typing import Any, Callable


@dataclass
class RequestContext:
    op: str
    instance_uuid: uuid.UUID
    params: dict[str, str]
    storage_dir: Path


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
    return {}


def handle_list(rctx: RequestContext) -> web.StreamResponse:
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
                          [sorted(versions.items())[-1][1]]}
            elif rctx.params['version'] == 'all':
                result = {rctx.params['record']:
                          [v for k, v in sorted(versions.items())]}
            else:
                if rctx.params['version'] in versions.values():
                    result = {rctx.params['record']: [rctx.params['version']]}
    elif 'version' in rctx.params and rctx.params['version'] != 'all':
        raise web.HTTPBadRequest(
            reason='list operation requires record or record and version or '
            'record and version="all", '
            'but got no record and version != "all".')
    else:
        for record_dir in rctx.storage_dir.iterdir():
            if 'version' in rctx.params:
                result[record_dir.name] = \
                    [v for k, v in sorted(all_versions(record_dir).items())]
            else:
                result[record_dir.name] = []
    return web.json_response(result)


def handle_not_implemented_yet(rctx: RequestContext) -> web.StreamResponse:
    return web.Response(text='Not implemented yet.')


@dataclass
class OpHandler:
    method: Any
    handler: Callable[[RequestContext], web.StreamResponse]


OPERATIONS: dict[str, OpHandler] = {
    'get': OpHandler(web.get, handle_not_implemented_yet),
    'put': OpHandler(web.put, handle_not_implemented_yet),
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
                          request.app['storage_dir'])
    check_request_context(rctx)
    return OPERATIONS[op].handler(rctx)


def main() -> None:
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
