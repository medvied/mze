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

from abc import ABC, abstractmethod
from typing import Optional, Any
from dataclasses import dataclass


# TODO define mze.api.Id, subclass it for every SomethingId
@dataclass
class ServiceId:
    sid: uuid.UUID


class Service(ABC):
    instance_id: ServiceId

    def __init__(self, *,
                 cfg: dict[str, Any],
                 argv: Optional[list[str]] = None,
                 environ: Optional[dict[str, str]] = None) -> None:
        if environ is not None:
            if 'MZE_INSTANCE_ID' in environ:
                self.instance_id.sid = uuid.UUID(environ['MZE_INSTANCE_ID'])
        # TODO handle --instance-id parameter from argv
        if 'MZE_INSTANCE_ID' in cfg:
            self.instance_id.sid = uuid.UUID(cfg['MZE_INSTANCE_ID'])

    @abstractmethod
    def run(self) -> None:
        pass
