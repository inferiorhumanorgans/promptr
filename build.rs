#[cfg(any(target_os = "macos", target_os = "freebsd"))]
fn build_ffi() {
    #[cfg(target_os = "macos")]
    let platform_file = "ffi/mac_os.cc";

    #[cfg(target_os = "freebsd")]
    let platform_file = "ffi/freebsd.cc";

    cxx_build::bridge("src/lib.rs")
        .flag_if_supported("-std=c++17")
        .file(platform_file)
        .compile("libpromptr");
}

fn main() {
    #[cfg(any(target_os = "macos", target_os = "freebsd"))]
    build_ffi();
}
