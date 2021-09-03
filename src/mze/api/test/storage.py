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

import mze.api


class TestStorage(unittest.TestCase, mze.api.Storage):
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
        self.init(self.cfg_init(0))
        self.destroy()
        self.post_destroy(0)
