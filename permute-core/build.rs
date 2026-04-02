fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let lib_base = std::path::Path::new(&manifest_dir).join("../libsndfile-binaries");
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    // Prefer the universal dylib when available — covers both native and universal builds.
    let universal = lib_base.join("libsndfile_universal.dylib");
    if universal.exists() {
        // Copy to OUT_DIR as libsndfile.dylib so the linker finds it by the standard name.
        let dst = std::path::Path::new(&out_dir).join("libsndfile.dylib");
        std::fs::copy(&universal, &dst).expect("failed to copy libsndfile_universal.dylib");
        println!("cargo:rustc-link-search={}", out_dir);
        println!("cargo:rustc-link-lib=dylib=sndfile");
        println!("cargo:rerun-if-changed={}", universal.display());
        return;
    }

    // Fall back to arch-specific dylibs.
    println!("cargo:rustc-link-search={}", lib_base.display());
    match target_arch.as_str() {
        "aarch64" => {
            println!("cargo:rustc-link-lib=dylib=sndfile");
            println!("cargo:rustc-link-search={}", lib_base.join("arm64").display());
        }
        "x86_64" => {
            println!("cargo:rustc-link-lib=dylib=sndfile");
            println!("cargo:rustc-link-search={}", lib_base.join("x86_64").display());
        }
        _ => {
            println!("cargo:rustc-link-lib=dylib=sndfile");
        }
    }

    println!("cargo:rerun-if-changed={}", lib_base.join("arm64/libsndfile.dylib").display());
    println!("cargo:rerun-if-changed={}", lib_base.join("x86_64/libsndfile.dylib").display());
}
