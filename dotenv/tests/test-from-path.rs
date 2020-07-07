mod common;

use dotenv::*;
use std::env;

use crate::common::*;

#[test]
fn test_from_path() {
    let dir = make_test_dotenv().unwrap();

    let mut path = env::current_dir().unwrap();
    path.push(".env");

    let _ = from_path(&path);

    assert_eq!(env::var("TESTKEY").unwrap(), "test_val");

    env::set_current_dir(dir.path().parent().unwrap()).unwrap();
    dir.close().unwrap();
}
