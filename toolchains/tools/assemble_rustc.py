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
import shutil
import subprocess
import sys

def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Given a list of Rust libraries, create a sysroot that can be used for the next round of bootstrap"
        ),
        fromfile_prefix_chars="@",
    )
    parser.add_argument(
        "--rustc",
        required=True,
        type=Path,
        help="path to rustc",
    )
    parser.add_argument(
        "--out-dir",
        required=True,
        type=Path,
        help="The compiler tree directory to write to",
    )
    parser.add_argument(
        "--target",
        required=True,
        type=str,
        help="Rust target platform",
    )
    parser.add_argument(
        "--llvm-lib",
        required=True,
        type=Path,
        help="Path to LLVM .so file",
    )

    return parser.parse_args()

def assemble_rustc(args: argparse.Namespace) -> None:
    root_dir = args.out_dir
    bin_dir = root_dir / "bin"
    bin_dir.mkdir(parents=True, exist_ok=True)

    lib_dir = root_dir / "lib"
    lib_dir.mkdir(parents=True, exist_ok=True)

    target_dir = root_dir / "lib" / "rustlib" / args.target
    target_dir.mkdir(parents=True, exist_ok=True)

    codegen_backends_dir = target_dir / "codegen-backends"
    codegen_backends_dir.mkdir(parents=True, exist_ok=True)

    llvm_link = lib_dir / args.llvm_lib.name
    llvm_dest = args.llvm_lib
    shutil.copy(llvm_dest, llvm_link)

    target_bin = target_dir / "bin"
    target_bin.mkdir(parents=True, exist_ok=True)
    shutil.copy(args.rustc, bin_dir / "rustc")
    return


def main() -> None:
    args = parse_args()
    assemble_rustc(args)

if __name__ == "__main__":
    main()
