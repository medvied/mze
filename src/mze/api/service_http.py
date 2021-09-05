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

from typing import Optional, Any
from aiohttp import web


class ServiceHTTP(Service):
    web_location: str

    def __init__(self, *,
                 cfg: dict[str, Any],
                 argv: Optional[list[str]] = None,
                 environ: Optional[dict[str, str]] = None) -> None:
        super().__init__(cfg=cfg, argv=argv, environ=environ)
        if environ is not None:
            if 'MZE_WEB_LOCATION' in environ:
                self.web_location = environ['MZE_WEB_LOCATION']
        # TODO handle --web-location parameter from argv
        if 'MZE_WEB_LOCATION' in cfg:
            self.web_location = cfg['MZE_WEB_LOCATION']

    def run(self) -> None:
        main_app = web.Application()
        main_app.add_subapp(self.web_location, self.app())
        web.run_app(main_app, port=80)

    def app(self) -> web.Application:
        application = web.Application()
        application['MZE_INSTANCE_ID'] = self.instance_id
        application['MZE_WEB_LOCATION'] = self.web_location
        # TODO add /health endpoint
        return application
