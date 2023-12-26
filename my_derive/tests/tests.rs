#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/derive/test_pass.rs");
    t.compile_fail("tests/ui/derive/test_failed.rs");
}
