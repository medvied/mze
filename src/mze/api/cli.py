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


import argparse

from abc import ABC, abstractmethod
from typing import Optional, Any


class CLI(ABC):
    """
    Intended use case:

    - call parse_argv_environ_all(os.argv, os.environ) first. It would execute
      parse_argv_environ(os.argv, os.environ) for all predecessors;
    - call run().
    """
    @abstractmethod
    def parse_cfg_argv_environ(self, cfg: dict[str, str], argv: list[str],
                               environ: dict[str, str]) -> None:
        # It's not raising NotImplementedError because all functions with this
        # name are being called from parse_cfg_argv_environ_all(), including
        # this one.
        pass

    @abstractmethod
    def run(self) -> None:
        raise NotImplementedError

    def parse_cfg_argv_environ_all(self, cfg: dict[str, str], argv: list[str],
                                   environ: dict[str, str]) -> None:
        c: Any
        for c in self.__class__.mro()[::-1]:
            if hasattr(c, 'parse_cfg_argv_environ'):
                c.parse_cfg_argv_environ(self, cfg, argv, environ)
                # mypy complains about the previous line:
                # error: "type" has no attribute "parse_cfg_argv_environ"
                # [attr-defined]
                #   c.parse_argv_environ(self, argv, environ)
                #   ^
                # It's a false positive because there is a check that the
                # function is actually present in the class.
                # Solution: disable type checks with "c: Any".

    def parse_cfg_argv_environ_single(
            self, cfg: dict[str, str], argv: list[str],
            environ: dict[str, str], *,
            key: Optional[str] = None,
            argv_flags: Optional[list[str]] = None,
            argv_kwargs: Optional[dict[str, Any]] = None,
            default_value: Any = None) -> Any:
        assert argv_kwargs is None or argv_flags is not None, \
            (argv_kwargs, argv_flags)
        if key is not None and key in cfg:
            return cfg[key]
        if argv_flags is not None:
            parser = argparse.ArgumentParser(prog='python -m mze',
                                             allow_abbrev=False)
            if argv_kwargs is None:
                store_action = parser.add_argument(*argv_flags)
            else:
                store_action = parser.add_argument(*argv_flags, **argv_kwargs)
            args, _ = parser.parse_known_args(args=argv)
            if (value := getattr(args, store_action.dest)) is not None:
                return value
        if key is not None and key in environ:
            return environ[key]
        return default_value
