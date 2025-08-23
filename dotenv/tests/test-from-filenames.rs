use std::env;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

use dotenv::*;

#[test]
fn test_from_filenames_priority() {
    let temp = TempDir::new().unwrap();
    let current = env::current_dir().unwrap();
    env::set_current_dir(temp.path()).unwrap();

    let mut env_file = File::create(".env").unwrap();
    env_file.write_all(b"TESTKEY=test_val\n").unwrap();
    env_file.sync_all().unwrap();

    // clear it to start fresh
    env::remove_var("TESTKEY");

    // should find .env when .env.local doesn't exist
    from_filenames(&[".env.local", ".env"]).unwrap();
    assert_eq!(env::var("TESTKEY").unwrap(), "test_val");

    let mut local_file = File::create(".env.local").unwrap();
    local_file.write_all(b"TESTKEY=local_val\n").unwrap();
    local_file.sync_all().unwrap();

    env::remove_var("TESTKEY");

    // should find .env.local first (higher priority)
    from_filenames(&[".env.local", ".env"]).unwrap();
    assert_eq!(env::var("TESTKEY").unwrap(), "local_val");

    // restore
    env::set_current_dir(&current).unwrap();
    temp.close().unwrap();
}
