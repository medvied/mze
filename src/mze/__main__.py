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


import os
import sys

import importlib

from typing import Any

# Without "mze: Any" mypy gives the following error:
#
# src/mze/__main__.py: note: In function "main":
# src/mze/__main__.py:27: error: Module has no attribute "__all__"
# [attr-defined]
#         assert class_name in mze.__all__, (class_name, mze.__all__)
#                              ^
# src/mze/__main__.py:28: error: Name "mze.api.CLI" is not defined
# [name-defined]
#         cli: mze.api.CLI = getattr(mze, class_name)()
#              ^
mze: Any
# Normal import couldn't be used here because we're at the top level of the
# module.
mze = importlib.import_module('.', 'mze')


def main() -> None:
    class_name = sys.argv[1]
    assert class_name in mze.__all__, (class_name, mze.__all__)
    cli: mze.api.CLI = getattr(mze, class_name)()
    cli.parse_cfg_argv_environ_all({}, sys.argv[2:], os.environ)
    cli.run()


if __name__ == '__main__':
    main()
