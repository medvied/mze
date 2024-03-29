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


import glob

from pathlib import Path

from plumbum import local  # type: ignore


def use_local_pypi() -> None:
    Path('/root/.config/pip').mkdir(parents=True)
    for f, target in [('pip.conf', '/root/.config/pip/pip.conf'),
                      ('.pypirc', '/root/.pypirc')]:
        print(f'Setting up {target}.', flush=True)
        ((local['envsubst'] < f'cicd/{f}.local.envsubst') > target)()


def python_file_list(s: str) -> list[str]:
    return [s for s in sorted(glob.glob(s))
            if not any([s.endswith(x) for x in [
                '/__pycache__',
            ]])]
