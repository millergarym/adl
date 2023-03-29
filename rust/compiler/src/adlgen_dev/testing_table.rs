// @generated from adl module testing_table

use serde::Deserialize;
use serde::Serialize;

pub type TestFilesMetaData = Vec<TestFileMetaData>;

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct TestFileMetaData {
  pub search_path: String,

  pub modules: Vec<String>,

  #[serde(default="TestFileMetaData::def_fail")]
  pub fail: bool,
}

impl TestFileMetaData {
  pub fn new(search_path: String, modules: Vec<String>) -> TestFileMetaData {
    TestFileMetaData {
      search_path: search_path,
      modules: modules,
      fail: TestFileMetaData::def_fail(),
    }
  }

  pub fn def_fail() -> bool {
    false
  }
}
