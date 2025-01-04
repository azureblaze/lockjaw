pub fn current_crate() -> String {
    std::env::var("CARGO_PKG_NAME").expect("missing pkg name env var")
}
