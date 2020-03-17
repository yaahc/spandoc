#[test]
fn no_std() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile/with_multi_await.rs");
    t.pass("tests/compile/with_single_await.rs");
}
