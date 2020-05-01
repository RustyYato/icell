use trybuild::TestCases;

#[test]
#[cfg_attr(miri, ignore)]
fn compile_fail() {
    let cases = TestCases::new();

    cases.compile_fail("tests/compile_fail/runtime/*.rs");
    cases.compile_fail("tests/compile_fail/scoped/*.rs");
    cases.compile_fail("tests/compile_fail/immovable/*.rs");
    cases.compile_fail("tests/compile_fail/typeid/*.rs");
    #[cfg(feature = "std")]
    cases.compile_fail("tests/compile_fail/typeid-tl/*.rs");
}
