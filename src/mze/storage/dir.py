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

import uuid
import logging
import pathlib

from collections.abc import Sequence
from urllib.parse import urlparse
from typing import Any, Optional

from mze.api.storage import BlobId, BlobInfo, BlobData, StorageClient


logger = logging.getLogger(__name__)


class StorageClientDir(StorageClient):
    """
    API

    - init()/fini()
    - create()/destroy()
    - fsck()
    - get()
    - put()
    - head()
    - catalog()
    - delete()
    """
    path: pathlib.Path

    def init(self, cfg: dict[str, Any]) -> None:
        logger.debug(f'{cfg=}')
        parse_result = urlparse(self.server_url)
        assert parse_result.scheme == 'file', (parse_result, self.server_url)
        self.path = pathlib.Path(parse_result.path)

    def fini(self) -> None:
        logger.debug(f'{self.path=}')

    def create(self, cfg: dict[str, Any]) -> None:
        logger.debug(f'{cfg=}')
        self.init(cfg)
        self.path.mkdir(parents=True)

    def destroy(self) -> None:
        logger.debug(f'{self.path=}')
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
        data.put_to_file(self.path / str(bid.bid))
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
