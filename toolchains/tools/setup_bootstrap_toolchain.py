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
from typing import Dict, Set, Tuple
import subprocess
import shutil
import tarfile
import tempfile

def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Construct the bootstrap compiler from stdlib and rustc tars"
        ),
        fromfile_prefix_chars="@",
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
        "--rustc-tar",
        required=False,
        type=Path,
        help="Path to bootstrap rustc tar",
    )

    parser.add_argument(
        "--stdlib-tar",
        required=False,
        type=Path,
        help="Path to bootstrap stdlib tar",
    )
    return parser.parse_args()

def setup_bootstrap_toolchain(args: argparse.Namespace) -> None:
    out_dir = args.out_dir
    target = args.target
    # Ugly but works, import_rustc.sh downloads stdlib and rustc tars. We unpack and construct a toolchain here
    with tempfile.TemporaryDirectory() as tmpdirname:
        stdlib_tar = tarfile.open(args.stdlib_tar)
        stdlib_skip = stdlib_tar.getmembers()[0].name + "/rust-std-" + target
        stdlib_tar.extractall(tmpdirname)
        stdlib_tar.close()

        shutil.copytree(
            tmpdirname + "/" + stdlib_skip + "/lib/rustlib/" + target,
            str(out_dir / "lib" / "rustlib" / target),
            dirs_exist_ok = True,
        )

        rustc_tar = tarfile.open(args.rustc_tar)
        rustc_skip = rustc_tar.getmembers()[0].name
        rustc_tar.extractall(tmpdirname)
        rustc_tar.close()

        shutil.copytree(
            tmpdirname + "/" + rustc_skip + "/rustc",
            str(out_dir),
            dirs_exist_ok = True,
        )

def main() -> None:
    args = parse_args()
    setup_bootstrap_toolchain(args)


if __name__ == "__main__":
    main()
