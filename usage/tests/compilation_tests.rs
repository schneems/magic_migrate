#[test]
fn should_not_compile() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/fails/*.rs");
}

#[test]
fn should_compile() {
    let t = trybuild::TestCases::new();
    t.pass("tests/pass/*.rs");
}
