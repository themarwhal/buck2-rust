## Setup
- Caveat: This build makes lots of assumptions about what target it runs on, so it probably only works for `x86_64-pc-windows-gnu` (it can be cleaned up)
- To get started run `import_rustc.sh`. This script will pull in the bootstrap compiler and ci-llvm artifacts that the build depends on.
  - The code under `library/src` and `compiler/src` are pulled in from https://github.com/rust-lang/rust with `import_rustc.sh`.
  - See `rust_src_version` to see which version the script was last ran with
- Pull in the `prelude` git submodule: `git submodule update --init prelude`
- Buck2 is currently configured to use whatever `clang++` and `python` available on your system path
- To setup `reindeer`, git clone https://github.com/facebookincubator/reindeer then `cargo install --path=.`
- To setup `buck2`, git clone https://github.com/facebookincubator/buck2 then `cargo install --path=cli --release`
  - [FB] On a devserver, I just build, not install, the binary and run it directly to not pollute my system path
- Run `buck2 run //test:hello_world_on_stage2` to run a hello_world with the bootstrapped rustc+libs

## How it works
- Conceptually, the bootstrap example in buck2 lays out the structure: https://github.com/facebookincubator/buck2/tree/main/examples/bootstrap
- `library/src/BUCK` and `compiler/src/BUCK` are generated entirely by `reindeer`
  - See `reindeer.toml` and `fixups` under each for `reindeer` configs
- Both `library/stdlib_rule_defs.bzl` and `compiler/compiler_rule_defs.bzl` have light logic around additional dependencies and rustc flags
- Most of the interesting targets to construct the toolchain for each stage live in `toolchains/BUCK`
