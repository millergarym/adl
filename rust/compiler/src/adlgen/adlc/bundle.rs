// @generated from adl module adlc.bundle

use serde::Deserialize;
use serde::Serialize;

/**
 * Expected to live in a file named `adl.bundle.json`
 */
#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct AdlBundle {
  pub path: String,

  #[serde(default="AdlBundle::def_global_alias")]
  pub global_alias: Option<String>,

  /**
   * Version
   */
  pub adlc: String,

  #[serde(default="AdlBundle::def_requires")]
  pub requires: Vec<Require>,

  #[serde(default="AdlBundle::def_excludes")]
  pub excludes: Vec<Exclude>,

  #[serde(default="AdlBundle::def_replaces")]
  pub replaces: Vec<Replace>,

  #[serde(default="AdlBundle::def_retracts")]
  pub retracts: Vec<Retract>,
}

impl AdlBundle {
  pub fn new(path: String, adlc: String) -> AdlBundle {
    AdlBundle {
      path: path,
      global_alias: AdlBundle::def_global_alias(),
      adlc: adlc,
      requires: AdlBundle::def_requires(),
      excludes: AdlBundle::def_excludes(),
      replaces: AdlBundle::def_replaces(),
      retracts: AdlBundle::def_retracts(),
    }
  }

  pub fn def_global_alias() -> Option<String> {
    None
  }

  pub fn def_requires() -> Vec<Require> {
    vec![]
  }

  pub fn def_excludes() -> Vec<Exclude> {
    vec![]
  }

  pub fn def_replaces() -> Vec<Replace> {
    vec![]
  }

  pub fn def_retracts() -> Vec<Retract> {
    vec![]
  }
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct Require {
  #[serde(rename="ref")]
  pub r#ref: BundleRef,

  #[serde(default="Require::def_version")]
  pub version: Option<String>,

  #[serde(default="Require::def_indirect")]
  pub indirect: bool,
}

impl Require {
  pub fn new(r#ref: BundleRef) -> Require {
    Require {
      r#ref: r#ref,
      version: Require::def_version(),
      indirect: Require::def_indirect(),
    }
  }

  pub fn def_version() -> Option<String> {
    None
  }

  pub fn def_indirect() -> bool {
    false
  }
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub enum BundleRef {
  #[serde(rename="path")]
  Path(String),

  #[serde(rename="alias")]
  Alias(String),
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct Exclude {
  pub path: String,

  pub version: String,
}

impl Exclude {
  pub fn new(path: String, version: String) -> Exclude {
    Exclude {
      path: path,
      version: version,
    }
  }
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct Replace {
  pub path: String,

  pub version: Option<String>,
}

impl Replace {
  pub fn new(path: String, version: Option<String>) -> Replace {
    Replace {
      path: path,
      version: version,
    }
  }
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct Retract {
  pub version: String,

  #[serde(default="Retract::def_comment")]
  pub comment: Option<String>,
}

impl Retract {
  pub fn new(version: String) -> Retract {
    Retract {
      version: version,
      comment: Retract::def_comment(),
    }
  }

  pub fn def_comment() -> Option<String> {
    None
  }
}
