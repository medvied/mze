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

from abc import ABC
from dataclasses import dataclass

from .cli import CLI


# TODO define mze.api.Id, subclass it for every SomethingId
@dataclass
class ServiceId:
    sid: uuid.UUID


class Service(CLI, ABC):
    instance_id: ServiceId

    def parse_cfg_argv_environ(self, cfg: dict[str, str], argv: list[str],
                               environ: dict[str, str]) -> None:
        self.parse_cfg_argv_environ_single(cfg, argv, environ,
                                           key='MZE_INSTANCE_ID',
                                           argv_flags=['--instance-id'])


class ServiceServer(Service):
    pass


class ServiceClient(Service):
    # TODO s/servier_url/service_uri/g
    server_url: str

    def parse_cfg_argv_environ(self, cfg: dict[str, str], argv: list[str],
                               environ: dict[str, str]) -> None:
        self.server_url = self.parse_cfg_argv_environ_single(
            cfg, argv, environ,
            key='MZE_SERVER_URL',
            argv_flags=['--server-url'])
