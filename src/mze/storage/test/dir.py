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

import pathlib
import tempfile

from typing import Any

from mze import StorageClientDir
from mze.api import StorageClient
from mze.api.test import TestStorageClient


class TestStorageClientDir(TestStorageClient):
    temp_dir: pathlib.Path

    def init_object(self, ii: int) -> StorageClient:
        storage_client: StorageClient = StorageClientDir()
        storage_client.parse_cfg_argv_environ_all(
            {'MZE_SERVER_URL': f'file:{self.ii_dir(ii)}'}, [], {})
        return storage_client

    def cfg_test(self) -> dict[str, Any]:
        return {
            'simple_N': 100,
            'simple_blob_size_max': 0x100,
        }

    def ii_dir(self, ii: int) -> pathlib.Path:
        if not hasattr(self, 'temp_dir'):
            self.temp_dir = pathlib.Path(tempfile.mkdtemp())
        return self.temp_dir / str(ii)

    def instance_index_max(self) -> int:
        return 0x1000000

    def cfg_create(self, ii: int) -> dict[str, Any]:
        return {}

    def cfg_init(self, ii: int) -> dict[str, Any]:
        return {}

    def pre_create(self, ii: int) -> None:
        self.assertFalse(self.ii_dir(ii).exists(), self.ii_dir(ii))

    def post_destroy(self, ii: int) -> None:
        self.assertFalse(self.ii_dir(ii).exists(), self.ii_dir(ii))

    def pre_init(self, ii: int) -> None:
        self.assertTrue(self.ii_dir(ii).exists(), self.ii_dir(ii))

    def post_fini(self, ii: int) -> None:
        self.assertTrue(self.ii_dir(ii).exists(), self.ii_dir(ii))
