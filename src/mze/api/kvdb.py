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
Design highlights

- all operations are vectored
"""


from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Optional


@dataclass
class DBTable:
    pass


@dataclass
class DBKey:
    pass


@dataclass
class DBValue:
    pass


@dataclass
class DBResult:
    pass


class KVDB(ABC):
    @abstractmethod
    def table_create(self, tables: list[DBTable]) -> list[DBResult]:
        raise NotImplementedError

    @abstractmethod
    def table_delete(self, tables: list[DBTable]) -> list[DBResult]:
        raise NotImplementedError

    @abstractmethod
    def table_catalog(self) -> list[DBTable]:
        raise NotImplementedError

    @abstractmethod
    def lookup(self, keys: list[tuple[DBTable, DBKey, int]]) -> \
            list[list[tuple[DBResult, Optional[DBValue]]]]:
        raise NotImplementedError

    @abstractmethod
    def insert(self, kvs: list[tuple[DBTable, DBKey, DBValue]], *,
               return_old_values: bool) -> \
            list[tuple[DBResult, Optional[DBValue]]]:
        raise NotImplementedError

    @abstractmethod
    def update(self, kvs: list[tuple[DBTable, DBKey, DBValue]], *,
               return_old_values: bool) -> \
            list[tuple[DBResult, Optional[DBValue]]]:
        raise NotImplementedError

    @abstractmethod
    def upsert(self, kvs: list[tuple[DBTable, DBKey, DBValue]], *,
               return_old_values: bool) -> \
            list[tuple[DBResult, Optional[DBValue]]]:
        raise NotImplementedError

    @abstractmethod
    def delete(self, keys: list[tuple[DBTable, DBKey]], *,
               return_old_values: bool) -> \
            list[tuple[DBResult, Optional[DBValue]]]:
        raise NotImplementedError
