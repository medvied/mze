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


from .cli import CLI

from .service import ServiceId
from .service import Service

from .store import Store

from .service_http import ServiceHTTP
from .service_http import ServiceHTTPServer
from .service_http import ServiceHTTPClient

from .storage import BlobId
from .storage import BlobInfo
from .storage import BlobData
from .storage import Storage
from .storage import StorageServer
from .storage import StorageClient

from .record import Record
from .record import RecordDB
from .record import RecordDBServer
from .record import RecordDBClient


__all__ = [
    'CLI',

    'ServiceId',
    'Service',

    'Store',

    'ServiceHTTP',
    'ServiceHTTPServer',
    'ServiceHTTPClient',

    'BlobId',
    'BlobInfo',
    'BlobData',
    'Storage',
    'StorageServer',
    'StorageClient',

    'Record',
    'RecordDB',
    'RecordDBServer',
    'RecordDBClient',
]
