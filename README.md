# Bootstrap rustc with Buck2

> **_NOTE:_** This is an experimental project

## Dependencies

- Buck2 for building: <https://github.com/facebookincubator/buck2>
  - <https://buck2.build/docs/getting_started/>
- `clang++` an `python` as this build assumes those are available on the path
- Reindeer for generating build files: <https://github.com/facebookincubator/reindeer>
  - Only needed if you wish to make changes to the generated build files

## Initial setup

- Caveat: This build makes lots of assumptions about what target it runs on, so it probably only works for `x86_64-unknown-linux-gnu` (it can be cleaned up)
- To get started run `import_rustc.sh`. This script will pull in the bootstrap compiler and ci-llvm artifacts that the build depends on.
  - The code under `library/src` and `compiler/src` are pulled in from <https://github.com/rust-lang/rust> with `import_rustc.sh --download-source`.
  - See `rust_src_version` to see which version the script was last ran with
- Pull in the `prelude` git submodule: `git submodule update --init prelude` or run `buck2 init --git`
- Run `buck2 run //test:hello_world_on_stage2` to run a hello_world with the bootstrapped rustc+libs, while building press `p` to enable verbose logging with configuration

## How it works

- Conceptually, the bootstrap example in buck2 does the same thing as this project: <https://github.com/facebookincubator/buck2/tree/main/examples/bootstrap>
- `library/src/BUCK` and `compiler/src/BUCK` are generated entirely by `reindeer`
  - See `reindeer.toml` and `fixups` under each for `reindeer` configs
- Both `library/stdlib_rule_defs.bzl` and `compiler/compiler_rule_defs.bzl` have light logic around additional dependencies and rustc flags
- Most of the interesting targets to construct the toolchain for each stage live in `toolchains/BUCK`
- The actual bootstrapping follows what is layed out in the rustc doc: <https://rustc-dev-guide.rust-lang.org/building/bootstrapping.html#stages-of-bootstrapping>
- Roughly we take these steps:
  1. `toolchains//:stage0_rust_toolchain` is constructed with the downloaded bootstrap compiler
  1. configure the rust toolchain to use `toolchains//:stage0_rust_toolchain` for the stage0 platform, defined in `bootstrap/platform/BUCK`
  1. with the stage0 platform, build libraries under `library/src`, which contains the standard library and its dependencies
  1. plug in the stage0 standard libraries and build `compiler/src`, which contains rustc and its dependencies
  1. take the stage0 rustc and construct `toolchains//:stage0_compiler_artifacts` which is a directory tree that mimics what a normal toolchain looks like. The important thing here is that `bin/rustc` exists. This will then plug into the toolchain used for stage1. This is used for the stage1 platform.
  1. build the standard libs again on stage1
  1. we now have a freshly built compiler with its standard libraries that can run `//test:hello_world_with_bootstrapped_compiler`
