fn main() {
    // Tell cargo to link against the wdsp library
    println!("cargo:rustc-link-lib=wdsp");

    // Specify the library path (if it's not in the default locations)
    println!("cargo:rustc-link-search=/usr/local/lib");
}
