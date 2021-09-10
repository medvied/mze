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

import json
import uuid
import logging
import requests

from collections.abc import Sequence
from typing import Optional, Any
from aiohttp import web

import mze.api
from mze.api import BlobId, BlobInfo, BlobData


logger = logging.getLogger(__name__)


class StorageServerHTTP(mze.api.StorageServer, mze.api.ServiceHTTPServer):
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
            id_info: Sequence[tuple[BlobId, Optional[BlobInfo]]] = []
            if op == 'put':
                assert len(bids) in [0, 1], (bids)
                bid = None if len(bids) == 0 else bids[0]
                id_info = self.put(blobs=[(
                    bid, BlobData(data=await request.read(), path=None))])
                assert len(id_info) == 1, id_info
            elif op == 'catalog':
                id_info = self.catalog()
            elif op in ['head', 'delete']:
                infos = self.head(bids) if op == 'head' else self.delete(bids)
                # TODO use strict for zip in Python 3.10
                id_info = [(bid, info) for bid, info in zip(bids, infos)]
            else:
                return web.HTTPNotImplemented()
            return web.Response(body=json.dumps(id_info))

    async def handler_management(self, request: web.Request, op: str) -> \
            web.StreamResponse:
        cfg: dict[str, Any] = {}
        if op in ['init', 'create']:
            assert request.can_read_body
            body = await request.json()
            assert body is dict, body
            for k, v in body:
                assert k is str, (k, body)
                cfg[k] = v
            (self.init if op == 'init' else self.create)(cfg)
        else:
            {
                'fini': self.fini,
                'destroy': self.destroy,
                'fsck': self.fsck,
            }[op]()
        # TODO error handling
        return web.HTTPOk()

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
    server_url: str

    def init_or_create(self, cfg: dict[str, Any], op: str) -> None:
        requests.get(f'{self.server_url}/{op}', data=cfg)

    def fini_or_destroy_or_fsck(self, op: str) -> None:
        requests.get(f'{self.server_url}/{op}')

    def init(self, cfg: dict[str, Any]) -> None:
        self.init_or_create(cfg, 'init')

    def fini(self) -> None:
        self.fini_or_destroy_or_fsck('fini')

    def create(self, cfg: dict[str, Any]) -> None:
        self.init_or_create(cfg, 'create')

    def destroy(self) -> None:
        self.fini_or_destroy_or_fsck('destroy')

    def fsck(self) -> None:
        self.fini_or_destroy_or_fsck('fsck')

    def get(self, bids: Sequence[BlobId]) -> list[Optional[BlobData]]:
        datas: list[Optional[BlobData]] = []
        for bid in bids:
            r = requests.get(f'{self.server_url}/get',
                             params={'id': str(bid.bid)})
            assert r.status_code in [200, 404]
            if r.status_code == 200:
                datas.append(BlobData(data=r.content, path=None))
            else:
                datas.append(None)
        return datas

    def put(self, blobs: Sequence[tuple[Optional[BlobId], BlobData]]) -> \
            list[tuple[BlobId, BlobInfo]]:
        ids_infos: list[tuple[BlobId, BlobInfo]] = []
        for bid, data in blobs:
            if data.path is not None:
                with open(data.path, 'rb') as f:
                    binary_data = f.read()
            elif data.data is not None:
                binary_data = data.data
            params = {} if bid is None else {'id': str(bid.bid)}
            r = requests.put(f'{self.server_url}/put',
                             data=binary_data, params=params)
            # TODO validate before returning to the user
            ids_infos.append(r.json()[0])
        return ids_infos

    def head_or_delete(self, op: str, bids: Sequence[BlobId]) -> \
            list[Optional[BlobInfo]]:
        url = f'{self.server_url}/{op}'
        ids = [str(bid.bid) for bid in bids]
        if op == 'head':
            r = requests.get(url, data=ids)
        else:
            r = requests.delete(url, data=ids)
        ids_infos = r.json()
        # TODO validate before returning
        return [info for _, info in ids_infos]

    def head(self, bids: Sequence[BlobId]) -> list[Optional[BlobInfo]]:
        return self.head_or_delete('head', bids)

    def catalog(self) -> list[tuple[BlobId, BlobInfo]]:
        r = requests.get(f'{self.server_url}/catalog')
        # TODO validate
        return [(bid, info) for bid, info in r.json()]

    def delete(self, bids: Sequence[BlobId]) -> list[Optional[BlobInfo]]:
        return self.head_or_delete('delete', bids)
