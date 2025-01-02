pub fn current_crate() -> String {
    std::env::var("CARGO_CRATE_NAME").expect("missing crate name env var")
}
