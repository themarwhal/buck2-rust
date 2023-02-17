load(
    "//third_party:rust_third_party.bzl",
    "third_party_rust_binary",
    "third_party_rust_library",
)
load(":lazy.bzl", "lazy")

_STAGE0_RUSTC_FLAGS = [
    # Point to an empty sysroot so we point to dependencies we've built, not from the bootstrap compiler
    "--cfg=bootstrap",
    "--check-cfg=values(bootstrap)",
]

# Used for all stages, mostly taken from snooping on what x.py ends up running from
_OTHER_RUSTC_FLAGS = [
    "--sysroot=$(location toolchains//tools:empty_sysroot)",

    # `-Z unstable-options` is needed to supply `noprelude` to `--extern`
    "-Zunstable-options",
    # Prevent Rust from adding `-nodefaultlibs` to linker invocations so we can
    # link against compiler-rt, libc, etc
    "-Cdefault-linker-libraries=y",
    "-C",
    "opt-level=3",
    "-Cembed-bitcode=no",
    "-Cdebuginfo=0",
    "-Zmacro-backtrace",
    "-Zbinary-dep-depinfo",

    # By default, rustc does not include unwind tables unless they are required
    # for a particular target. They are not required by RISC-V targets, but
    # compiling the standard library with them means that users can get
    # backtraces without having to recompile the standard library themselves.
    "-Cforce-unwind-tables=yes",
]

_RUSTC_FLAGS = select({
    "bootstrap//:stage0": _STAGE0_RUSTC_FLAGS + _OTHER_RUSTC_FLAGS,
    "bootstrap//:stage1": _OTHER_RUSTC_FLAGS,
    "bootstrap//:stage2": _OTHER_RUSTC_FLAGS,
})

_ADDITIONAL_DEPS = {
    # Seems to only work if we use this named_deps method for alloc and core
    "alloc": ":rustc-std-workspace-alloc-1.99.0",
    "core": ":rustc-std-workspace-core-1.99.0",
    "compiler_builtins": ":compiler_builtins",
}

# Can pull this into reindeer config
_TARGETS_THAT_NEED_ADDITIONAL_DEPS = [
    "memchr",
    "object",
    "std",
]

_USE_BOOTSTRAP_COMPILER = [
    # Targets that should be unaffected by target linkage strategy, etc
    # For versioned targets, include the name up to the hyphen. Example:
    #    cc-1.0.73 -> cc-
    "cc-",
]

_DYLIB_FRIENDLY_TARGETS = [
    "std",
    "test",
]

_BOOTSTRAP_COMPILER = "$(location toolchains//:bootstrap_compiler)"

def stdlib_rust_library(name, **kwargs):
    env = kwargs.get("env", {})
    rustc_flags = kwargs.get("rustc_flags", [])
    named_deps = kwargs.get("named_deps", {})
    deps = kwargs.get("deps", [])

    # See https://github.com/rust-lang/rust/blob/master/src/bootstrap/builder.rs#L1654
    # For running build scripts, we need a full compiler. So always use the stage0 one
    if lazy.is_any(lambda use_bootstrap: use_bootstrap in name, _USE_BOOTSTRAP_COMPILER):
        print("Using stage0 compiler for", name)

        # HACKY, what we want is a functional rust_bootstrap_library
        env["USE_THIS_RUSTC_DIR"] = _BOOTSTRAP_COMPILER
    else:
        # Otherwise set sysroot, -L, BOOTSTRAP_ENV and all that jazz
        rustc_flags = rustc_flags + _RUSTC_FLAGS
        env["RUSTC_BOOTSTRAP"] = "1"

        if lazy.is_all(lambda dylib_name: not name.startswith(dylib_name), _DYLIB_FRIENDLY_TARGETS):
            kwargs["preferred_linkage"] = "static"

        if lazy.is_any(lambda needs_core_name: name.startswith(needs_core_name), _TARGETS_THAT_NEED_ADDITIONAL_DEPS):
            # Would be nice to move this to reindeer, but we need named_deps
            print("Adding extra deps=core,alloc,compiler_builtins for", name)
            named_deps.update(_ADDITIONAL_DEPS)

    kwargs["rustc_flags"] = rustc_flags
    kwargs["named_deps"] = named_deps
    kwargs["deps"] = deps

    third_party_rust_library(name, env = env, **kwargs)

def stdlib_rust_binary(name, **kwargs):
    # The only rust binaries in library should be buildscripts, which require the bootstrap compiler
    # See https://github.com/rust-lang/rust/blob/master/src/bootstrap/builder.rs#L1654
    env = kwargs.get("env", {})

    print("Using stage0 compiler for", name)

    # HACKY, what we want is a functional rust_bootstrap_binary
    env["USE_THIS_RUSTC_DIR"] = _BOOTSTRAP_COMPILER
    third_party_rust_binary(name, env = env, **kwargs)

def add_platformed_stdlib(platform):
    targets = [
        "//library:core",
        "//library:std",
        "//library:compiler_builtins",
        "//library:alloc",
        "//library:panic_unwind",
        "//library:proc_macro",
    ]

    def _get_name_with_stage(target, platform):
        name = target.split(":")[-1]
        stage = platform.split(":")[-1]
        return stage + "_" + name

    [
        native.configured_alias(
            name = _get_name_with_stage(target, platform),
            actual = target,
            platform = platform,
            visibility = ["PUBLIC"],
        )
        for target in targets
    ]
