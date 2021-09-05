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

"""

endpoint  input     output      quantity      HTTP

/put      id, data  info        one -> one    PUT
/get      id        info, data  one -> one    GET
/head     id        id, info    many -> many  GET
/catalog  -         id, info    one -> many   GET
/delete   id        info        many -> many  DELETE

/init     cfg       ok/fail                   POST
/fini     -         ok/fail                   POST
/create   cfg       ok/fail                   POST
/destroy  -         ok/fail                   POST
/fsck     -         ok/fail                   POST
"""

import uuid
import logging

from typing import Optional, Any
from aiohttp import web

import mze.api
from mze.api import BlobId, BlobInfo, BlobData


logger = logging.getLogger(__name__)


class StorageServerHTTP(mze.api.StorageServer, mze.api.ServiceHTTP):
    ENDPOINTS: dict[str, Any] = {
        'put': web.put,
        'get': web.get,
        'head': web.get,
        'catalog': web.get,
        'delete': web.delete,

        'init': web.post,
        'fini': web.post,
        'create': web.post,
        'destroy': web.post,
        'fsck': web.post,
    }

    async def handler_data(self, request: web.Request, op: str) -> \
            web.StreamResponse:
        bids: list[BlobId] = []
        query = {k: v for k, v in request.query.items()}
        if op in ['get', 'put'] and 'id' in query:
            bids.append(BlobId(bid=uuid.UUID(query['id'])))
        if op in ['head', 'delete']:
            assert request.can_read_body
            for single_id in await request.json():
                bids.append(BlobId(bid=uuid.UUID(single_id)))
        if op == 'get':
            assert len(bids) == 1, (bids)
            datas = self.get(bids)
            assert len(datas) == 1, datas
            data = datas[0]
            if data is not None:
                if data.path is not None:
                    return web.FileResponse(path=data.path)
                if data.data is not None:
                    return web.Response(body=data.data)
                return web.HTTPInternalServerError()
            else:
                return web.HTTPNotFound()
        else:
            id_info: list[Optional[tuple[BlobInfo, BlobData]]] = []
            print(id_info)  # XXX placeholder
        return web.Response(text='Hi')

    async def handler_management(self, request: web.Request, op: str) -> \
            web.StreamResponse:
        return web.Response(text='Hi')

    async def handler(self, request: web.Request) -> web.StreamResponse:
        application = request.app
        assert request.path.startswith(application['MZE_WEB_LOCATION']), \
            f'{request.path=} {application["MZE_WEB_LOCATION"]=}'
        op = request.path[len(request.app['web_location']):]
        if op[0] == '/':
            op = op[1:]
        assert op in self.ENDPOINTS.keys(), f'{op=} {self.ENDPOINTS.keys()=}'
        if op in ['put', 'get', 'head', 'catalog', 'delete']:
            return await self.handler_data(request, op)
        else:
            return await self.handler_management(request, op)

    def app(self) -> web.Application:
        application = super().app()
        application.add_routes([
            method(f'/{endpoint}', self.handler)
            for endpoint, method in self.ENDPOINTS.items()])
        return application


class StorageClientHTTP(mze.api.StorageClient):
    pass
