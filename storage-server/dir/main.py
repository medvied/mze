#!/usr/bin/env python3


import os

from aiohttp import web


async def index_html(request: web.Request) -> web.StreamResponse:
    return web.Response(text='Hello, world!')


def main() -> None:
    app = web.Application()

    storage_server_dir = web.Application()
    storage_server_dir.add_routes([
        web.get('/', index_html)
    ])
    app.add_subapp(os.environ['MZE_WEB_LOCATION'], storage_server_dir)
    web.run_app(app, port=80)


if __name__ == '__main__':
    main()
