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


from .service import Service

from aiohttp import web


class ServiceHTTP(Service):
    pass


class ServiceHTTPServer(ServiceHTTP):
    client_url: str
    web_location: str

    def parse_cfg_argv_environ(self, cfg: dict[str, str], argv: list[str],
                               environ: dict[str, str]) -> None:
        self.client_url = self.parse_cfg_argv_environ_single(
            cfg, argv, environ,
            key='MZE_CLIENT_URL',
            argv_flags=['--client-url'])
        self.web_location = self.parse_cfg_argv_environ_single(
            cfg, argv, environ,
            key='MZE_WEB_LOCATION',
            argv_flags=['--web-location'])

    def run(self) -> None:
        main_app = web.Application()
        main_app.add_subapp(self.web_location, self.app())
        web.run_app(main_app, port=80)

    # TODO rename app() -> application()
    # This would allow to create shorter variables everywhere a web.Application
    # is needed
    def app(self) -> web.Application:
        application = web.Application()
        application['MZE_INSTANCE_ID'] = self.instance_id
        application['MZE_WEB_LOCATION'] = self.web_location
        # TODO add /health endpoint
        return application


class ServiceHTTPClient(ServiceHTTP):
    server_url: str

    def parse_cfg_argv_environ(self, cfg: dict[str, str], argv: list[str],
                               environ: dict[str, str]) -> None:
        self.server_url = self.parse_cfg_argv_environ_single(
            cfg, argv, environ,
            key='MZE_SERVER_URL',
            argv_flags=['--server-url'])
