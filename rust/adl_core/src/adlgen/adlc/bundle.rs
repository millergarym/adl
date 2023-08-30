// @generated from adl module adlc.bundle

use serde::Deserialize;
use serde::Serialize;

/**
 * Expected to live in a file named `adl.bundle.json`
 */
#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct AdlBundle {
  pub bundle: String,

  pub module_prefix: Option<String>,

  /**
   * Version
   */
  pub adlc: String,

  #[serde(default="AdlBundle::def_npm_opts")]
  pub npm_opts: Option<NpmOptions>,

  #[serde(default="AdlBundle::def_requires")]
  pub requires: Vec<RequireBundle>,

  #[serde(default="AdlBundle::def_local_requires")]
  pub local_requires: Vec<String>,

  #[serde(default="AdlBundle::def_excludes")]
  pub excludes: Vec<Exclude>,

  #[serde(default="AdlBundle::def_replaces")]
  pub replaces: Vec<Replace>,

  #[serde(default="AdlBundle::def_retracts")]
  pub retracts: Vec<Retract>,
}

impl AdlBundle {
  pub fn new(bundle: String, module_prefix: Option<String>, adlc: String) -> AdlBundle {
    AdlBundle {
      bundle: bundle,
      module_prefix: module_prefix,
      adlc: adlc,
      npm_opts: AdlBundle::def_npm_opts(),
      requires: AdlBundle::def_requires(),
      local_requires: AdlBundle::def_local_requires(),
      excludes: AdlBundle::def_excludes(),
      replaces: AdlBundle::def_replaces(),
      retracts: AdlBundle::def_retracts(),
    }
  }

  pub fn def_npm_opts() -> Option<NpmOptions> {
    None
  }

  pub fn def_requires() -> Vec<RequireBundle> {
    vec![]
  }

  pub fn def_local_requires() -> Vec<String> {
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
pub struct NpmOptions {
  pub pkg_name: String,

  #[serde(default="NpmOptions::def_version")]
  pub version: String,
}

impl NpmOptions {
  pub fn new(pkg_name: String) -> NpmOptions {
    NpmOptions {
      pkg_name: pkg_name,
      version: NpmOptions::def_version(),
    }
  }

  pub fn def_version() -> String {
    "1.0.0".to_string()
  }
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct RequireBundle {
  pub bundle: String,

  #[serde(default="RequireBundle::def_version")]
  pub version: Option<String>,

  #[serde(default="RequireBundle::def_indirect")]
  pub indirect: bool,
}

impl RequireBundle {
  pub fn new(bundle: String) -> RequireBundle {
    RequireBundle {
      bundle: bundle,
      version: RequireBundle::def_version(),
      indirect: RequireBundle::def_indirect(),
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
