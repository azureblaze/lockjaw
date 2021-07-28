#[test]
fn duplicated_root_epilogue() {
    let t = trybuild::TestCases::new();
    t.compile_failed_with(
        "tests/src/src.rs",
        vec!["crate 'dep' which already called lockjaw::epilogue!(root)"],
    )
}
