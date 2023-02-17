load("@prelude//rust:rust_toolchain.bzl", "RustPlatformInfo", "RustToolchainInfo")
load(
    "@root//third_party:rust_third_party.bzl",
    "get_native_host_triple",
)

target = get_native_host_triple()

def _rust_toolchain_impl(ctx):
    return [
        DefaultInfo(),
        RustToolchainInfo(
            clippy_driver = "clippy-driver",
            compiler = ctx.attrs.compiler[RunInfo],
            default_edition = ctx.attrs.default_edition,
            extern_html_root_url_prefix = ctx.attrs.extern_html_root_url_prefix,
            failure_filter_action = ctx.attrs.failure_filter_action[RunInfo],
            rustc_action = ctx.attrs.rustc_action[RunInfo],
            rustc_flags = ctx.attrs.rustc_flags,
            rustc_target_triple = ctx.attrs.rustc_target_triple,
            rustdoc = "rustdoc",
            rustdoc_flags = ctx.attrs.rustdoc_flags,
            rustdoc_test_with_resources = ctx.attrs.rustdoc_test_with_resources[RunInfo],
        ),
        RustPlatformInfo(
            name = "x86_64",
        ),
    ]

rust_toolchain = rule(
    impl = _rust_toolchain_impl,
    attrs = {
        "compiler": attrs.dep(providers = [RunInfo]),
        "default_edition": attrs.option(attrs.string(), default = None),
        "extern_html_root_url_prefix": attrs.option(attrs.string(), default = None),
        "failure_filter_action": attrs.default_only(attrs.dep(providers = [RunInfo], default = "prelude//rust/tools:failure_filter_action")),
        "rustc_action": attrs.default_only(attrs.dep(providers = [RunInfo], default = "prelude//rust/tools:rustc_action")),
        "rustc_flags": attrs.list(attrs.arg(), default = []),
        "rustc_target_triple": attrs.string(default = target),
        "rustdoc_flags": attrs.list(attrs.string(), default = []),
        "rustdoc_test_with_resources": attrs.default_only(attrs.dep(providers = [RunInfo], default = "prelude//rust/tools:rustdoc_test_with_resources")),
    },
    is_toolchain_rule = True,
)

def _bootstrap_compiler_impl(ctx):
    out = ctx.actions.declare_output("bootstrap")

    cmd = cmd_args(
        [
            ctx.attrs.setup_bootstrap_toolchain[RunInfo],
            "--out-dir",
            out.as_output(),
            "--rustc-tar",
            ctx.attrs.rustc_tar,
            "--stdlib-tar",
            ctx.attrs.stdlib_tar,
            "--target",
            get_native_host_triple(),
        ],
    )
    ctx.actions.run(cmd, category = "setup_bootstrap_compiler")
    return [DefaultInfo(default_output = out)]

bootstrap_compiler = rule(
    impl = _bootstrap_compiler_impl,
    attrs = {
        "rustc_tar": attrs.source(),
        "stdlib_tar": attrs.source(),
        "setup_bootstrap_toolchain": attrs.default_only(attrs.dep(providers = [RunInfo], default = "//tools:setup_bootstrap_toolchain")),
    },
)
