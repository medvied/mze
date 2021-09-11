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


from collections.abc import Sequence
from typing import Any

from mze.api.record import Record, RecordDBClient


class RecordDBClientSQLite(RecordDBClient):
    def init(self, cfg: dict[str, Any]) -> None:
        pass

    def fini(self) -> None:
        pass

    def create(self, cfg: dict[str, Any]) -> None:
        pass

    def destroy(self) -> None:
        pass

    def fsck(self) -> None:
        pass

    async def insert(self, rs: Sequence[Record]) -> list[Record]:
        pass

    async def select(self, rs: Sequence[tuple[Record, int]]) \
            -> list[list[Record]]:
        pass

    async def update(self, rs: Sequence[Record]) -> list[Record]:
        pass

    async def delete(self, rs: Sequence[Record], *,
                     return_old_values: bool) -> list[list[Record]]:
        pass
