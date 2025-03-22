fn main() {
    // Get the target architecture
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();

    // Add the search path for the binaries
    println!(r"cargo:rustc-link-search=../libsndfile-binaries");

    // Link against the appropriate library based on architecture
    match target_arch.as_str() {
        "aarch64" => {
            println!(r"cargo:rustc-link-lib=dylib=sndfile");
            println!(r"cargo:rustc-link-search=../libsndfile-binaries/arm64");
        }
        "x86_64" => {
            println!(r"cargo:rustc-link-lib=dylib=sndfile");
            println!(r"cargo:rustc-link-search=../libsndfile-binaries/x86_64");
        }
        _ => {
            println!(r"cargo:rustc-link-lib=dylib=sndfile");
        }
    }

    // Ensure we rebuild if any of these files change
    println!("cargo:rerun-if-changed=../libsndfile-binaries/arm64/libsndfile.dylib");
    println!("cargo:rerun-if-changed=../libsndfile-binaries/x86_64/libsndfile.dylib");
}
