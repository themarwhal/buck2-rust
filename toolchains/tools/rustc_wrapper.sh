#!/bin/bash

# A little hacky but this is the easiest way without changing the prelude or writing my own rustc_action.py.
# For the toolchain, we construct a sysroot directory and rustc lives in sysroot/bin/rustc
# $1 contains all the args and the first of those is the path to the actual compiler (added by the command_alias this script is passed around with)

# So what we need to do is
# 1. find the actual path to rustc, which is the first element in the argsfile
# 2. pop the first line from argsfile
# 3. invoke rustc with modified argsfile

args_file=${1//'@'/''}
sysroot=$(head -n 1 $args_file)

# Remove the rustc path from args file
tail -n +2 "$args_file" > "$args_file.tmp"

if [[ -z "${USE_THIS_RUSTC_DIR}" ]]; then
  LD_LIBRARY_PATH="$sysroot/lib" "$sysroot/bin/rustc" "@$args_file.tmp"
else
  LD_LIBRARY_PATH="$USE_THIS_RUSTC_DIR/lib" "$USE_THIS_RUSTC_DIR/bin/rustc" "@$args_file.tmp"
fi
