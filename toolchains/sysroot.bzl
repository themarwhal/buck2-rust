load(
    "@prelude//rust:link_info.bzl",
    "RustLinkInfo",
)
load(
    "@root//third_party:rust_third_party.bzl",
    "get_native_host_triple",
)

target = get_native_host_triple()

def _get_all_libs(p):
    libs = {}

    default_out = p[DefaultInfo].sub_targets["static_pic"][DefaultInfo].default_outputs[0]
    shared_lib = p[DefaultInfo].sub_targets["shared"][DefaultInfo].default_outputs[0]

    # No set?
    libs[default_out] = None
    libs[shared_lib] = None

    rust_link_info = p[RustLinkInfo].styles

    # get [1] to grab static_pic, maybe there's a better way to do this
    # didn't seem to work if I just searched for "static_pic" key
    static_libs = rust_link_info[rust_link_info.keys()[1]]

    [libs.update({l: None}) for l in static_libs.transitive_deps.keys()]

    return libs.keys()

def _compiler_and_stdlibs_impl(ctx):
    out = ctx.actions.declare_output(ctx.attrs.name)

    all_libs = []
    std_transitive_rdeps = [
        all_libs.extend(_get_all_libs(lib))
        for lib in ctx.attrs.stdlibs
    ]

    cmd = cmd_args(
        [
            ctx.attrs.assemble_rustc_and_stdlibs[RunInfo],
            "--out-dir",
            out.as_output(),
            "--target",
            target,
            "--libs",
            all_libs,
            "--rustc",
            ctx.attrs.rustc_dir[DefaultInfo].default_outputs[0],
        ],
    )
    ctx.actions.run(cmd, category = "make_compiler_and_stdlibs")
    return [DefaultInfo(default_output = out)]

compiler_and_stdlibs = rule(
    impl = _compiler_and_stdlibs_impl,
    attrs = {
        "rustc_dir": attrs.dep(),
        "stdlibs": attrs.list(attrs.dep()),
        "assemble_rustc_and_stdlibs": attrs.default_only(attrs.dep(providers = [RunInfo], default = "//tools:assemble_rustc_and_stdlibs")),
    },
)

def _compiler_artifacts_impl(ctx):
    out = ctx.actions.declare_output("compiler_artifacts")

    rustc_path = ctx.attrs.rustc[DefaultInfo].default_outputs[0]
    llvm_lib = ctx.attrs.llvm_lib[DefaultInfo].default_outputs[0]

    cmd = cmd_args([
        ctx.attrs.assemble_rustc[RunInfo],
        "--out-dir",
        out.as_output(),
        "--rustc",
        rustc_path,
        "--target",
        target,
        "--llvm-lib",
        llvm_lib,
    ])

    ctx.actions.run(cmd, category = "make_compiler_artifacts")

    return [DefaultInfo(default_output = out)]

compiler_artifacts = rule(
    impl = _compiler_artifacts_impl,
    attrs = {
        "rustc": attrs.dep(),
        "assemble_rustc": attrs.default_only(attrs.dep(providers = [RunInfo], default = "//tools:assemble_rustc")),
        "llvm_lib": attrs.default_only(attrs.dep(providers = [DefaultInfo], default = "root//compiler/ci-llvm:lib")),
    },
)

def add_platformed_stdlib(targets, platform):
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
