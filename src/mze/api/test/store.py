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


import unittest

from abc import abstractmethod
from typing import Any

from mze.api import Store


class TestStore(unittest.TestCase):
    @abstractmethod
    def init_object(self, ii: int) -> Store:
        raise NotImplementedError

    @abstractmethod
    def cfg_test(self) -> dict[str, Any]:
        raise NotImplementedError

    @abstractmethod
    def instance_index_max(self) -> int:
        """
        Is not used right now.

        Use case: in case if the amount of instances is limited (example: each
        instance requires a TCP/IP port and # of ports is limited) this value
        will limit the number of concurrently created instaces of the class
        under test.
        """
        raise NotImplementedError

    @abstractmethod
    def cfg_create(self, ii: int) -> dict[str, Any]:
        raise NotImplementedError

    @abstractmethod
    def cfg_init(self, ii: int) -> dict[str, Any]:
        raise NotImplementedError

    @abstractmethod
    def pre_create(self, ii: int) -> None:
        raise NotImplementedError

    @abstractmethod
    def post_destroy(self, ii: int) -> None:
        raise NotImplementedError

    @abstractmethod
    def pre_init(self, ii: int) -> None:
        raise NotImplementedError

    @abstractmethod
    def post_fini(self, ii: int) -> None:
        raise NotImplementedError

    def test_create_destroy(self) -> None:
        store = self.init_object(0)
        cfg = self.cfg_create(0)
        self.pre_create(0)
        store.create(cfg)
        store.destroy()
        self.post_destroy(0)

    def test_init_fini(self) -> None:
        store = self.init_object(0)
        self.pre_create(0)
        store.create(self.cfg_create(0))
        store.fini()
        self.post_fini(0)
        self.pre_init(0)
        store.init(self.cfg_init(0))
        store.fini()
        self.post_fini(0)
        store = self.init_object(0)
        self.pre_init(0)
        store.init(self.cfg_init(0))
        store.destroy()
        self.post_destroy(0)
