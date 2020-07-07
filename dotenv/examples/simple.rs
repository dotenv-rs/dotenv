use dotenv::dotenv;
use std::env;

fn main() {
    let _ = dotenv();

    for (key, value) in env::vars() {
        println!("{}: {}", key, value);
    }
}
