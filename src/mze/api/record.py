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

from collections.abc import Sequence
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Optional

from .service import ServiceServer, ServiceClient
from .store import Store
from .cli import CLI


@dataclass
class Record:
    rid: Optional[uuid.UUID]
    tags: Optional[list[str]]
    attrs: Optional[dict[str, str]]
    version: Optional[int]


class RecordDB(Store, ABC):
    @abstractmethod
    async def insert(self, rs: Sequence[Record]) -> list[Record]:
        pass

    @abstractmethod
    async def select(self, rs: Sequence[tuple[Record, int]]) \
            -> list[list[Record]]:
        pass

    @abstractmethod
    async def update(self, rs: Sequence[Record]) -> list[Record]:
        pass

    @abstractmethod
    async def delete(self, rs: Sequence[Record], *,
                     return_old_values: bool) -> list[list[Record]]:
        pass


class RecordDBServer(RecordDB, ServiceServer, CLI):
    pass


class RecordDBClient(RecordDB, ServiceClient, CLI):
    pass
