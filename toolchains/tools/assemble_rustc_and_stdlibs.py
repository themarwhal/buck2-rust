#!/usr/bin/env python3
# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under both the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree and the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree.

import argparse
import os
from pathlib import Path
from typing import Dict, Set, Tuple, List
import subprocess
import shutil
import tarfile
import tempfile

def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Given a list of Rust libraries, create a sysroot that can be used for the next round of bootstrap"
        ),
        fromfile_prefix_chars="@",
    )
    parser.add_argument(
        "--libs",
        nargs='+',
        dest="libs",
        default=[],
        help="List of libs that should be symlinked into stdlib artifact tree",
    )

    parser.add_argument(
        "--out-dir",
        required=True,
        type=Path,
        help="The link tree directory to write to",
    )

    parser.add_argument(
        "--target",
        required=False,
        type=str,
        help="Rust target platform",
    )

    parser.add_argument(
        "--rustc",
        required=False,
        type=Path,
        help="Path to rustc",
    )

    return parser.parse_args()


def assemble_compiler_and_stdlib(args):
    args.out_dir.mkdir(parents = True, exist_ok=True)

    lib_dir =  args.out_dir /  "lib"
    bin_dir =  args.out_dir / "bin"
    target_dir = lib_dir / "rustlib" / args.target
    rustlib_lib_dir = target_dir / "lib"

    shutil.copytree(args.rustc / "lib", lib_dir, dirs_exist_ok=True)
    shutil.copytree(args.rustc / "bin", bin_dir, dirs_exist_ok=True)

    rustlib_lib_dir.mkdir(parents = True, exist_ok=True)

    for lib in args.libs:
        lib = Path(lib)
        shutil.copy(lib, rustlib_lib_dir / lib.name)

def main() -> None:
    args = parse_args()
    assemble_compiler_and_stdlib(args)


if __name__ == "__main__":
    main()
