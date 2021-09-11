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
TODO test put() with BlobId=None
TODO test fsck()
TODO random test: make a random deterministic (defined by seed) sequence of
put()s and delete()s and run it. Check catalog(), get() and head() after each
element of the sequence.
"""


import uuid
import random
import pathlib
import tempfile
import unittest

from abc import abstractmethod

from mze.api import StorageClient
from mze.api.storage import BlobId, BlobData

from .store import TestStore


class TestStorageClient(TestStore, unittest.TestCase):
    storage_client: StorageClient

    @abstractmethod
    def init_object(self, ii: int) -> StorageClient:
        raise NotImplementedError

    def check_presence(self, supposedly_present: list[bool],
                       test_ids: list[BlobId], test_data: list[bytes],
                       one_by_one: bool) -> None:
        N = len(test_ids)
        if one_by_one:
            infos = [self.storage_client.head([test_ids[i]])[0]
                     for i in range(N)]
        else:
            infos = self.storage_client.head(test_ids)
        self.assertEqual(len(infos), N)
        for i, info in enumerate(infos):
            if info is None:
                self.assertFalse(supposedly_present[i])
            else:
                self.assertEqual(info.size, len(test_data[i]))
        c = self.storage_client.catalog()
        c_bids = sorted([bid for bid, _ in c], key=lambda bid: bid.bid)
        p_bids = sorted([bid for i, bid in enumerate(test_ids)
                         if supposedly_present[i]], key=lambda bid: bid.bid)
        self.assertEqual(c_bids, p_bids)
        c_infos = {bid.bid: info for bid, info in c}
        for i in range(N):
            if supposedly_present[i]:
                self.assertEqual(len(test_data[i]),
                                 c_infos[test_ids[i].bid].size)

    def put_data(self, to_put: list[bool], present: list[bool],
                 test_ids: list[BlobId],
                 test_data: list[bytes], tmpdir: pathlib.Path) -> None:
        N = len(test_ids)
        for i in range(N):
            self.assertFalse(to_put[i] and present[i], f'{i=}')
        blobs = []
        sizes = []
        for i in range(N):
            if to_put[i]:
                filename = tmpdir / str(i)
                if filename.exists():
                    blob_data = BlobData(data=None, path=filename)
                else:
                    blob_data = BlobData(data=test_data[i], path=None)
                blobs.append((test_ids[i], blob_data))
                sizes.append(len(test_data[i]))
        ids_infos = self.storage_client.put(blobs=blobs)
        for j in range(len(blobs)):
            bid, info = ids_infos[j]
            self.assertEqual(bid.bid, blobs[j][0].bid)
            self.assertEqual(info.size, sizes[j])
        for i in range(N):
            if to_put[i]:
                present[i] = True

    def check_data(self, supposedly_present: list[bool],
                   test_ids: list[BlobId], test_data: list[bytes],
                   one_by_one: bool) -> None:
        N = len(test_ids)
        if one_by_one:
            datas = [self.storage_client.get([bid])[0] for bid in test_ids]
        else:
            datas = self.storage_client.get(test_ids)
        self.assertEqual(len(datas), N)
        for i, data in enumerate(datas):
            if data is None:
                self.assertFalse(supposedly_present[i])
            else:
                self.assertEqual(data.get_bytes(), test_data[i])

    def test_simple(self) -> None:
        N: int = self.cfg_test()['simple_N']
        blob_size_max = self.cfg_test()['simple_blob_size_max']

        self.storage_client = self.init_object(0)

        # prepare test data and files
        test_data = [random.randbytes(random.randrange(blob_size_max + 1))
                     for i in range(N)]
        test_ids = [BlobId(bid=uuid.uuid4()) for i in range(N)]
        present = [False for i in range(N)]
        tmpdir = pathlib.Path(tempfile.mkdtemp())
        for i in range(0, N, 2):
            with open(tmpdir / str(i), 'wb') as f:
                f.write(test_data[i])

        self.pre_create(0)
        self.storage_client.create(self.cfg_create(0))

        # test put(), get(), head(), catalog()
        # the first N//5 * 4 blobs are put one by one,
        # the rest are put all at once
        for i in range(N):
            if i == 0:
                self.check_presence(present, test_ids, test_data, False)
            if i == N-1:
                self.put_data([j >= N//5 * 4 for j in range(N)],
                              present, test_ids, test_data, tmpdir)
            elif i < N//5 * 4:
                self.put_data([j == i for j in range(N)],
                              present, test_ids, test_data, tmpdir)
            else:
                continue
            self.check_presence(present, test_ids, test_data, i % (N//6) == 0)
            if i % (N//4) == 0 or i == N-1:
                self.check_data(present, test_ids, test_data, i in [0, N-1])

        # test delete()
        for i in range(N):
            info = self.storage_client.delete([test_ids[i]])
            present[i] = False
            self.assertEqual(len(info), 1)
            self.assertIsNot(info[0], None)
            assert info[0] is not None  # to make mypy happy
            self.assertEqual(info[0].size, len(test_data[i]))
            self.check_presence(present, test_ids, test_data, i % (N//7) == 0)

        self.storage_client.destroy()
        self.post_destroy(0)

        # remove test data and files
        for i in range(N):
            (tmpdir / str(i)).unlink(missing_ok=True)
        tmpdir.rmdir()
