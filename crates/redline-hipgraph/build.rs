// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

use std::process::{Command, Stdio};

/// Is the mold linker usable on this machine?
///
/// mold is a link-*speed* optimization, never a correctness requirement, so it
/// must stay opportunistic. Emitting `-fuse-ld=mold` unconditionally made this
/// crate unbuildable for anyone without mold on PATH — gcc reports the missing
/// linker as the thoroughly misleading `collect2: fatal error: cannot find 'ld'`,
/// which points at binutils rather than at the real cause. That broke every
/// stock CI runner and every fresh clone, including the llama.cpp integration
/// path that loads this cdylib.
fn mold_available() -> bool {
    Command::new("mold")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn main() {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by Cargo");
    let version_script = std::path::Path::new(&manifest_dir).join("redline_hipgraph.map");

    println!("cargo:rerun-if-changed={}", version_script.display());
    println!("cargo:rerun-if-env-changed=REDLINE_USE_MOLD");

    // REDLINE_USE_MOLD: `0` forces the default linker even where mold exists
    // (useful when bisecting a link-order problem); `1` forces the flag on and
    // lets the link fail loudly if mold is absent; unset autodetects.
    let use_mold = match std::env::var("REDLINE_USE_MOLD").as_deref() {
        Ok("0") => false,
        Ok("1") => true,
        _ => mold_available(),
    };
    if use_mold {
        println!("cargo:rustc-cdylib-link-arg=-fuse-ld=mold");
    }

    println!(
        "cargo:rustc-cdylib-link-arg=-Wl,--version-script={}",
        version_script.display()
    );
    println!("cargo:rustc-link-arg-cdylib=-Wl,-soname,libredline_hipgraph.so");
}
