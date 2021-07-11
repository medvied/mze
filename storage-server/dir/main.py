#!/usr/bin/env python3

"""C.ss.dir: a directory is served directly over HTTP

This storage server has a directory where all the data is located and it
provides HTTP API to serve files from that directory.
"""

import os
import uuid

from aiohttp import web
from typing import Any


OPERATIONS: dict[str, Any] = {
    'get': web.get,
    'put': web.put,
    'head': web.head,
    'list': web.get,
    'delete': web.delete,
}


def validate_params(instance_uuid: str, params: dict[str, str]) -> None:
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
            raise web.HTTPBadRequest(reason=f'uuid.UUID("{v}") failed: '
                                     f'{repr(e)}.')


async def handler(request: web.Request) -> web.StreamResponse:
    instance_uuid = request.app['instance_uuid']
    params = {k: v for k, v in request.query.items()}
    print(f'{params=}', flush=True)
    validate_params(instance_uuid, params)
    return web.Response(text='Hello, world!')


def main() -> None:
    app = web.Application()

    storage_server_dir = web.Application()
    for op, method in OPERATIONS.items():
        storage_server_dir.add_routes([method(f'/{op}', handler)])
    app.add_subapp(os.environ['MZE_WEB_LOCATION'], storage_server_dir)
    storage_server_dir['instance_uuid'] = os.environ['MZE_INSTANCE_UUID']
    storage_server_dir['storage_dir'] = os.environ['MZE_STORAGE_DIR']
    web.run_app(app, port=80)


if __name__ == '__main__':
    main()
