use std::fs::{self, File};
use std::io::BufReader;
use std::path::PathBuf;

use serde::Deserialize;

use crate::adlgen_dev::testing_table::{TestFilesMetaData, TestFileMetaData};

use super::*;

#[test]
fn generate_ts_from_test_files() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("../../adl/tests/testing_table.json");

    let file = File::open(d).expect(&format!(
        "Failed to read file: {}",
        "../../adl/tests/testing_table.json"
    ));
    let reader = BufReader::new(file);

    let mut de = serde_json::Deserializer::from_reader(reader);
    match TestFilesMetaData::deserialize(&mut de) {
      Ok(tests) => { dbg!(tests); },
      Err(err) => assert!(false, "error deserializing testing_table {}", err),
    }

    // // Read the JSON contents of the file as an instance of `User`.
    // let u: Result<TestFilesMetaData, _> = serde_json::from_reader(reader);
    // match u {
    //     Ok(tests) => { dbg!(tests); },
    //     Err(err) => assert!(false, "error deserializing testing_table {}", err),
    // }
}
