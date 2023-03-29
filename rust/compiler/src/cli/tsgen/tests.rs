use std::fs::{self, File};
use std::io::BufReader;
use std::path::PathBuf;

use crate::adlgen_dev::testing_table::TestFilesMetaData;

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

    // let mut de = serde_json::Deserializer::from_reader(reader);
    // match TestFilesMetaData::deserialize(&mut de) {

    // }

    // Read the JSON contents of the file as an instance of `User`.
    let u: Result<TestFilesMetaData, _> = serde_json::from_reader(reader);
    match u {
        Ok(tests) => { dbg!(tests); },
        Err(err) => assert!(false, "error deserializing testing_table {}", err),
    }
}

// fn inp(s: &str) -> Input<'_> {
//     LocatedSpan::new(s)
// }

// fn assert_parse_eq<T>(pr: Res<Input<'_>, T>, v: T)
// where
//     T: std::fmt::Debug + PartialEq,
// {
//     match pr {
//         Ok((i, pv)) => {
//             assert_eq!(pv, v);
//             assert!(i.is_empty());
//         }
//         Err(e) => {
//             panic!("Unexpected parse failure: {}", e);
//         }
//     }
// }

// fn assert_parse_eq_2<T>(pr: Res<Input<'_>, T>, v: T, remaining: &str)
// where
//     T: std::fmt::Debug + PartialEq,
// {
//     match pr {
//         Ok((i, pv)) => {
//             assert_eq!(pv, v);
//             assert_eq!(*i.fragment(), remaining);
//         }
//         Err(e) => {
//             panic!("Unexpected parse failure: {}", e);
//         }
//     }
// }

// fn assert_module_file_ok(path: &str) {
//     let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
//     d.push(path);
//     let content = fs::read_to_string(d).expect(&format!("Failed to read file: {}", path));
//     let content_str: &str = &content;
//     let parse_result = raw_module(inp(content_str));
//     let err = parse_result.err().and_then(|e| match e {
//         Err::Error(e) => Some(e),
//         Err::Failure(e) => Some(e),
//         Err::Incomplete(_e) => None,
//     });

//     assert_eq!(err, Option::None);
// }
