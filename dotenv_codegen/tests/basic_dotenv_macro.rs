#[macro_use]
extern crate dotenv_codegen;

#[test]
fn dotenv_works() {
    assert_eq!(dotenv!("CODEGEN_TEST_VAR1"), "hello!");
}

#[test]
fn two_argument_form_works() {
    assert_eq!(
        dotenv!(
            "CODEGEN_TEST_VAR2",
            "err, you should be running this in the 'dotenv_codegen' \
             directory to pick up the right .env file."
        ),
        "'quotes within quotes'"
    );
}

#[test]
fn dotenv_or_default_works() {
    let default_value: &str = dotenv_or_default!("CODEGEN_TEST_NONEXISTING_VARIABLE", "hello!");
    assert_eq!(default_value, "hello!");
}

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
