#!/usr/bin/env python3

"""C.ss.dir: a directory is served directly over HTTP

This storage server has a directory where all the data is located and it
provides HTTP API to serve files from that directory.

Storage format:

object_uuid/version_number-version-uuid

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
    instance_uuid: uuid.UUID
    params: dict[str, str]


def check_params(instance_uuid: uuid.UUID, params: dict[str, str]) -> None:
    for k, v in params.items():
        if k not in ['instance', 'object', 'version']:
            raise web.HTTPBadRequest(
                reason=f'{k} is not one of ["instance", "object", "version"]')
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
        if k == 'instance' and v != str(instance_uuid):
            raise web.HTTPBadRequest(
                reason=f'The request is for instance {v}, '
                f'but the current instance is {instance_uuid}.')


def handle_list(rctx: RequestContext) -> web.StreamResponse:
    return web.Response(text='Hello, world!')


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
    check_params(instance_uuid, params)
    req_ctx = RequestContext(instance_uuid, params)
    return OPERATIONS[op].handler(req_ctx)


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
