use dotenv;
use std::env;

fn main() {
    // Try to load from multiple files in priority order
    // This will load from .env.local first, then fall back to .env if .env.local doesn't exist
    match dotenv::from_filenames(&[".env.local", ".env"]) {
        Ok(path) => println!("Loaded environment from: {:?}", path),
        Err(e) => println!("Failed to load any environment file: {}", e),
    }

    // Now you can use environment variables as usual
    match env::var("MY_VAR") {
        Ok(val) => println!("MY_VAR = {}", val),
        Err(_) => println!("MY_VAR not found in environment"),
    }
}
