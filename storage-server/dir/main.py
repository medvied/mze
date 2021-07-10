#!/usr/bin/env python3

"""C.ss.dir: a directory is served directly over HTTP

This storage server has a directory where all the data is located and it
provides HTTP API to serve files from that directory.
"""

import os

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
    for k in params:
        if k not in ['instance', 'object', 'version']:
            raise web.HTTPBadRequest()
    if params.get('instance') is not None:
        if params['instance'] not in [instance_uuid, 'any', 'all']:
            raise web.HTTPBadRequest()


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
