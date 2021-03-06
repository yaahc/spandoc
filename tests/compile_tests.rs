#[test]
fn no_std() {
    let t = trybuild::TestCases::new();
    t.pass("tests/compile/with_multi_await.rs");
    t.pass("tests/compile/with_single_await.rs");
    t.pass("tests/compile/with_valid_span_args.rs");
    t.pass("tests/compile/empty_blocks.rs");
    t.compile_fail("tests/compile/with_invalid_span_args.rs");
}
