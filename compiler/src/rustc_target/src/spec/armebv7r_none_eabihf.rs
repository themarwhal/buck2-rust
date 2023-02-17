// Targets the Cortex-R4F/R5F processor (ARMv7-R)

use crate::abi::Endian;
use crate::spec::{Cc, LinkerFlavor, Lld, PanicStrategy, RelocModel, Target, TargetOptions};

pub fn target() -> Target {
    Target {
        llvm_target: "armebv7r-unknown-none-eabihf".into(),
        pointer_width: 32,
        data_layout: "E-m:e-p:32:32-Fi8-i64:64-v128:64:128-a:0:32-n32-S64".into(),
        arch: "arm".into(),
        options: TargetOptions {
            abi: "eabihf".into(),
            endian: Endian::Big,
            linker_flavor: LinkerFlavor::Gnu(Cc::No, Lld::Yes),
            linker: Some("rust-lld".into()),
            relocation_model: RelocModel::Static,
            panic_strategy: PanicStrategy::Abort,
            features: "+vfp3,-d32,-fp16".into(),
            max_atomic_width: Some(32),
            emit_debug_gdb_scripts: false,
            // GCC and Clang default to 8 for arm-none here
            c_enum_min_bits: 8,
            ..Default::default()
        },
    }
}
