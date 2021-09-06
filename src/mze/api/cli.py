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


from abc import ABC, abstractmethod


class CLI(ABC):
    """
    Intended use case:
    - call parse_argv_environ_all(os.argv, os.environ) first. It would execute
      parse_argv_environ(os.argv, os.environ) for all predecessors;
    - call run().
    """
    @abstractmethod
    def parse_argv_environ(self, argv: list[str],
                           environ: dict[str, str]) -> None:
        pass

    @abstractmethod
    def run(self) -> None:
        pass

    def parse_argv_environ_all(self, argv: list[str],
                               environ: dict[str, str]) -> None:
        for c in self.__class__.mro()[::-1]:
            if 'parse_argv_environ' in vars(c):
                c.parse_argv_environ(self, argv, environ)  # type: ignore
                # mypy complains about the previous line:
                # error: "type" has no attribute "parse_argv_environ"
                # [attr-defined]
                #   c.parse_argv_environ(self, argv, environ)
                #   ^
                # It's a false positive because there is a check that the
                # function is actually present in the class.
