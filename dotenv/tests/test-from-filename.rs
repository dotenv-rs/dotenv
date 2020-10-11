mod common;

use dotenv::*;
use std::env;

use crate::common::*;

#[test]
fn test_from_filename() {
    let dir = make_test_dotenv().unwrap();

    let _ = from_filename(".env");

    assert_eq!(env::var("TESTKEY").unwrap(), "test_val");

    env::set_current_dir(dir.path().parent().unwrap()).unwrap();
    dir.close().unwrap();
}
