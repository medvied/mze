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


from .service import Service, ServiceServer, ServiceClient

from aiohttp import web


class ServiceHTTP(Service):
    pass


class ServiceHTTPServer(ServiceHTTP, ServiceServer):
    web_location: str

    def parse_cfg_argv_environ(self, cfg: dict[str, str], argv: list[str],
                               environ: dict[str, str]) -> None:
        self.web_location = self.parse_cfg_argv_environ_single(
            cfg, argv, environ,
            key='MZE_WEB_LOCATION',
            argv_flags=['--web-location'])

    def run(self) -> None:
        main_app = web.Application()
        main_app.add_subapp(self.web_location, self.application())
        web.run_app(main_app, port=80)

    def application(self) -> web.Application:
        application = web.Application()
        application['MZE_INSTANCE_ID'] = self.instance_id
        application['MZE_WEB_LOCATION'] = self.web_location
        # TODO add /health endpoint
        return application


class ServiceHTTPClient(ServiceHTTP, ServiceClient):
    pass
