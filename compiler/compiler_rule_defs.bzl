load(
    "@root//third_party:rust_third_party.bzl",
    "get_native_host_triple",
    "rust_buildscript_genrule_args",
    "rust_buildscript_genrule_srcs",
    "third_party_rust_binary",
    "third_party_rust_cxx_library",
    "third_party_rust_library",
)
load("//library:lazy.bzl", "lazy")

_OTHER_RUSTC_FLAGS = [
    # We inject stdlib targets so clear the sysroot
    "--sysroot=$(location toolchains//tools:empty_sysroot)",

    # `-Z unstable-options` is needed to supply `noprelude` to `--extern`
    "-Zunstable-options",
    # Prevent Rust from adding `-nodefaultlibs` to linker invocations so we can
    # link against compiler-rt, libc, etc
    "-Cdefault-linker-libraries=y",
    "-C",
    "opt-level=3",
    "-Cembed-bitcode=no",

    # By default, rustc does not include unwind tables unless they are required
    # for a particular target. They are not required by RISC-V targets, but
    # compiling the standard library with them means that users can get
    # backtraces without having to recompile the standard library themselves.
    "-Cforce-unwind-tables=yes",
    "-Clink-args=-Wl,-z,origin",
    "-Clink-args=-Wl,-rpath,$ORIGIN/../lib",
    "-Clink-args=-Wl,-lc",  # fixes c func linking error
    "-Clink-args=-Wl,-ldl",  # fixes c func linking error
    "-Clink-args=-Wl,-lm",  # fixes floating linking error
    "-Clink-args=-Wl,-lpthread",  # fixes floating linking error
]

_RUSTC_FLAGS = select({
    "bootstrap//:stage0": ["--cfg=bootstrap"] + _OTHER_RUSTC_FLAGS,
    "bootstrap//:stage1": _OTHER_RUSTC_FLAGS,
})

_DEPS = select({
    "bootstrap//:stage0": [
        "//library:stage0_std",
        "//library:stage0_core",
        "//library:stage0_compiler_builtins",
        "//library:stage0_alloc",
        "//library:stage0_panic_unwind",
        "//library:stage0_proc_macro",
    ],
    "bootstrap//:stage1": [
        "//library:stage1_std",
        "//library:stage1_core",
        "//library:stage1_compiler_builtins",
        "//library:stage1_alloc",
        "//library:stage1_panic_unwind",
        "//library:stage1_proc_macro",
    ],
})

ENVS = {
    "RUSTC_BOOTSTRAP": "1",
    "RUSTC_INSTALL_BINDIR": "",
    "CFG_COMPILER_HOST_TRIPLE": get_native_host_triple(),
    # Taking these values directly from what x.py seems to do
    "LLVM_LINK_SHARED": "1",
    "CFG_DEFAULT_CODEGEN_BACKEND": "llvm",
}

# Don't hard code these but it seems stdlibs and compiler version must match
NEEDS_VERSION_ENVS = ["rustc_interface", "rustc_passes"]
VERSION_ENVS = {
    "CFG_RELEASE": "1.67.1-dev",
    "CFG_RELEASE_CHANNEL": "dev",
    "CFG_VERSION": "1.67.1-dev",
}

FEATURES = select({
    # "panic-unwind", "backtrace", "compiler-builtins-c"
    "bootstrap//:stage0": ["llvm"],
    "bootstrap//:stage1": ["llvm"],
    "DEFAULT": [],  # Pick a better default
})

def rustc_compiler():
    rust_compiler_binary(
        name = "rustc",
        srcs = [
            "src/rustc/src/main.rs",
            "src/rustc/build.rs",
        ],
        deps = [
            ":rustc_driver",
            ":rustc_codegen_ssa",
            ":rustc_codegen_llvm",
            ":rustc_smir",
            "//compiler/ci-llvm:lib",
        ],
        visibility = ["PUBLIC"],
    )

def rust_compiler_library(name, **kwargs):
    env = kwargs.get("env", {})
    env.update(ENVS)
    if lazy.is_any(lambda needs_version_envs_name: name.startswith(needs_version_envs_name), NEEDS_VERSION_ENVS):
        env.update(VERSION_ENVS)
    kwargs["env"] = env

    kwargs["deps"] = kwargs.get("deps", []) + _DEPS
    kwargs["features"] = kwargs.get("features", []) + FEATURES
    kwargs["rustc_flags"] = kwargs.get("rustc_flags", []) + _RUSTC_FLAGS

    third_party_rust_library(name, **kwargs)

def rust_compiler_binary(name, **kwargs):
    env = kwargs.get("env", {})
    env.update(ENVS)
    kwargs["env"] = env

    kwargs["deps"] = kwargs.get("deps", []) + _DEPS
    kwargs["features"] = kwargs.get("features", []) + FEATURES
    kwargs["rustc_flags"] = kwargs.get("rustc_flags", []) + _RUSTC_FLAGS

    third_party_rust_binary(name, **kwargs)

def rust_compiler_buildscript_genrule_args(name, **kwargs):
    rust_buildscript_genrule_args(name, **kwargs)

def rust_compiler_buildscript_genrule_srcs(name, **kwargs):
    rust_buildscript_genrule_srcs(name, **kwargs)

def third_party_rust_cxx_library(name, **kwargs):
    native.cxx_library(name = name, **kwargs)
