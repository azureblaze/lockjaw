pub fn current_package() -> String {
    std::env::var("CARGO_PKG_NAME")
        .expect("missing pkg name env var")
        .replace("-", "_")
}
pub fn current_crate() -> String {
    std::env::var("CARGO_CRATE_NAME")
        .expect("missing crate name env var")
        .replace("-", "_")
}
