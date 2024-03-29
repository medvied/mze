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


from mze.storage.dir import StorageClientDir
from mze.storage.http import StorageServerHTTP
from mze.storage.http import StorageClientHTTP
from mze.storage.combinations import StorageServerHTTPClientDir

__all__ = [
    'StorageClientDir',
    'StorageServerHTTP',
    'StorageClientHTTP',
    'StorageServerHTTPClientDir',
]
