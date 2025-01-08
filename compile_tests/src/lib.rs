#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub fn set_src_path(path: &str) -> &str {
    std::env::set_var(
        "LOCKJAW_TRYBUILD_PATH",
        format!("{}/{}", std::env::var("CARGO_MANIFEST_DIR").unwrap(), path),
    );
    path
}
