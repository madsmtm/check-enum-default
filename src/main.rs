use std::{collections::BTreeMap, path::Path, process::Command};

use clang::{Clang, Index, TypeKind};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
struct TargetSpec {
    #[serde(rename = "llvm-target")]
    llvm_target: String,
    #[serde(default)]
    arch: String,
    #[serde(default)]
    os: String,
    #[serde(default)]
    env: String,
    // #[serde(default)]
    // abi: String,
    #[serde(flatten)]
    _rest: BTreeMap<String, Value>,
}

fn main() {
    clang_sys::load().unwrap();

    let clang = Clang::new().unwrap();
    let index = Index::new(&clang, true, true);

    let targets = get_targets();

    for (target_name, spec) in targets {
        println!("checking {target_name}");

        // Not supported by my version of Clang.
        if spec.llvm_target == "wasm32-linux-muslwali" || spec.arch == "xtensa" {
            continue;
        }

        // Uses the wrong LLVM target:
        // https://github.com/rust-lang/rust/pull/132570
        if target_name == "i686-unknown-uefi" && spec.llvm_target == "i686-unknown-windows-gnu" {
            continue;
        }

        let tu = index
            .parser(Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("enums.c"))
            .visit_implicit_attributes(true)
            .arguments(&["-target", &spec.llvm_target])
            .parse()
            .unwrap();

        for diag in tu.get_diagnostics() {
            eprintln!("{diag}");
        }

        let children = tu.get_entity().get_children();

        // We need to basically re-implement the logic in:
        // https://github.com/llvm/llvm-project/blob/llvmorg-23.1.0/clang/lib/Sema/SemaDecl.cpp#L18121-L18163
        let enum_variant = if (spec.os == "windows" && spec.env == "msvc") || spec.os == "uefi" {
            TypeKind::Int
        } else if spec.arch == "hexagon" {
            TypeKind::UChar
        } else {
            TypeKind::UInt
        };
        let signed_enum_variant = if spec.arch == "hexagon" {
            TypeKind::SChar
        } else {
            TypeKind::Int
        };

        let entity = &children[0];
        assert_eq!(entity.get_name().unwrap(), "some_enum");
        let ty = entity.get_enum_underlying_type().unwrap();
        assert_eq!(ty.get_kind(), enum_variant, "{spec:#?}");

        let entity = &children[1];
        assert_eq!(entity.get_name().unwrap(), "with_negative_variant");
        let ty = entity.get_enum_underlying_type().unwrap();
        assert_eq!(ty.get_kind(), signed_enum_variant, "{spec:#?}");

        let entity = &children[2];
        assert_eq!(entity.get_name().unwrap(), "explicit");
        let ty = entity.get_enum_underlying_type().unwrap();
        assert_eq!(ty.get_kind(), TypeKind::Int, "{spec:#?}");
    }
}

fn get_targets() -> BTreeMap<String, TargetSpec> {
    let output = Command::new("rustc")
        .arg("+nightly")
        .arg("--print=all-target-specs-json")
        .arg("-Zunstable-options")
        .output()
        .unwrap();

    serde_json::from_slice(&output.stdout).unwrap()
}
