#[macro_use]
extern crate dotenv_codegen;

pub fn main() {
    dotenv!("a", "b", "c");
}