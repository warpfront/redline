// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: 2026 Kaden Schutt <kaden@hipfire.dev>

fn main() {
    let manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by Cargo");
    let version_script = std::path::Path::new(&manifest_dir).join("redline_hipgraph.map");

    println!("cargo:rerun-if-changed={}", version_script.display());
    println!("cargo:rustc-cdylib-link-arg=-fuse-ld=mold");
    println!(
        "cargo:rustc-cdylib-link-arg=-Wl,--version-script={}",
        version_script.display()
    );
    println!("cargo:rustc-link-arg-cdylib=-Wl,-soname,libredline_hipgraph.so");
}
