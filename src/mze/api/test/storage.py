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


import uuid
import random
import pathlib
import tempfile
import unittest

from abc import abstractmethod
from typing import Any

import mze.api
from mze.api.storage import BlobId, BlobInfo, BlobData


class TestStorage(unittest.TestCase, mze.api.Storage):
    @abstractmethod
    def cfg_test(self) -> dict[str, Any]:
        pass

    @abstractmethod
    def instance_index_max(self) -> int:
        """
        Is not used right now.

        Use case: in case if the amount of instances is limited (example: each
        instance requires a TCP/IP port and # of ports is limited) this value
        will limit the number of concurrently created instaces of the class
        under test.
        """
        pass

    @abstractmethod
    def cfg_create(self, ii: int) -> dict[str, Any]:
        pass

    @abstractmethod
    def cfg_init(self, ii: int) -> dict[str, Any]:
        pass

    @abstractmethod
    def pre_create(self, ii: int) -> None:
        pass

    @abstractmethod
    def post_destroy(self, ii: int) -> None:
        pass

    @abstractmethod
    def pre_init(self, ii: int) -> None:
        pass

    @abstractmethod
    def post_fini(self, ii: int) -> None:
        pass

    def test_create_destroy(self) -> None:
        cfg = self.cfg_create(0)
        self.pre_create(0)
        self.create(cfg)
        self.destroy()
        self.post_destroy(0)

    def test_init_fini(self) -> None:
        self.pre_create(0)
        self.create(self.cfg_create(0))
        self.fini()
        self.post_fini(0)
        self.pre_init(0)
        self.init(self.cfg_init(0))
        self.destroy()
        self.post_destroy(0)

    def check_presence(self, supposedly_present: list[bool],
                       test_ids: list[uuid.UUID], one_by_one: bool) -> None:
        pass

    def put_data(self, to_put: list[bool], present: list[bool],
                 test_ids: list[uuid.UUID],
                 test_data: list[bytes], tmpdir: pathlib.Path) -> None:
        N = len(present)
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
                blobs.append((BlobId(bid=test_ids[i]),
                              BlobInfo(size=len(test_data[i]), info={}),
                              blob_data))
                sizes.append(len(test_data[i]))
        infos = self.put(blobs=blobs)
        for j in range(len(blobs)):
            self.assertEqual(infos[j].size, sizes[j])
        for i in range(N):
            if to_put[i]:
                present[i] = True

    def check_data(self, supposedly_present: list[bool],
                   test_ids: list[uuid.UUID]) -> None:
        pass

    def test_simple(self) -> None:
        N: int = self.cfg_test()['simple_N']
        blob_size_max = self.cfg_test()['simple_blob_size_max']

        # prepare test data and files
        test_data = [random.randbytes(random.randrange(blob_size_max + 1))
                     for i in range(N)]
        test_ids = [uuid.uuid4() for i in range(N)]
        present = [False for i in range(N)]
        tmpdir = pathlib.Path(tempfile.mkdtemp())
        for i in range(0, N, 2):
            with open(tmpdir / str(i), 'wb') as f:
                f.write(test_data[i])

        self.pre_create(0)
        self.create(self.cfg_create(0))

        # test put(), get(), head(), catalog()
        # the first N//5 * 4 blobs are put one by one,
        # the rest are put all at once
        for i in range(N):
            if i == 0:
                self.check_presence(present, test_ids, False)
            if i == N-1:
                self.put_data([j >= N//5 * 4 for j in range(N)],
                              present, test_ids, test_data, tmpdir)
            elif i < N//5 * 4:
                self.put_data([j == i for j in range(N)],
                              present, test_ids, test_data, tmpdir)
            else:
                continue
            self.check_presence(present, test_ids, i % (N//6) == 0)
            if i % (N//4) == 0 or i == N-1:
                self.check_data(present, test_ids)

        # test delete()
        for i in range(N):
            info = self.delete([BlobId(bid=test_ids[i])])
            present[i] = False
            self.assertEqual(len(info), 1)
            self.assertIsNot(info[0], None)
            assert info[0] is not None  # to make mypy happy
            self.assertEqual(info[0].size, len(test_data[i]))
            self.check_presence(present, test_ids, i % (N//7) == 0)

        self.destroy()
        self.post_destroy(0)

        # remove test data and files
        for i in range(N):
            (tmpdir / str(i)).unlink(missing_ok=True)
        tmpdir.rmdir()
