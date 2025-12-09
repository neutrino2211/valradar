fn main() {
    // Check if the target OS is macOS
    if cfg!(target_os = "macos") {
        // Set environment variable for macOS
        unsafe {
            std::env::set_var("PYO3_USE_ABI3_FORWARD_COMPATIBILITY", "1");
        };

        // Make it available during compilation
        println!("cargo:rustc-env=PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1");
    }
}
