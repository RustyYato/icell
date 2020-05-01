use trybuild::TestCases;

#[test]
fn compile_fail() {
    let cases = TestCases::new();

    cases.compile_fail("tests/compile_fail/*/*.rs");
}
