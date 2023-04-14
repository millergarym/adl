// @generated from adl module adlc.packaging

use serde::Deserialize;
use serde::Serialize;

/**
 * Expected to live in a file named `adl.work.json`
 */
#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct AdlWorkspace {
  /**
   * Version
   */
  pub adlc: String,

  #[serde(rename="use")]
  pub r#use: AdlPackageRefs,
}

impl AdlWorkspace {
  pub fn new(adlc: String, r#use: AdlPackageRefs) -> AdlWorkspace {
    AdlWorkspace {
      adlc: adlc,
      r#use: r#use,
    }
  }
}

pub type AdlPackageRefs = Vec<AdlPackageRef>;

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct AdlPackageRef {
  /**
   * must be a path to a directory directly under the folder containing the `adl.work.json` file.
   */
  pub root: String,

  #[serde(rename="genOptions")]
  pub gen_options: Vec<GenOptions>,
}

impl AdlPackageRef {
  pub fn new(root: String, gen_options: Vec<GenOptions>) -> AdlPackageRef {
    AdlPackageRef {
      root: root,
      gen_options: gen_options,
    }
  }
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub enum GenOptions {
  #[serde(rename="tsgen")]
  Tsgen(TypescriptGenOptions),
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct TypescriptGenOptions {
  #[serde(default="TypescriptGenOptions::def_referenceable")]
  pub referenceable: ReferenceableScopeOption,

  pub outputs: OutputOpts,

  #[serde(rename="includeRuntime")]
  pub include_runtime: bool,

  #[serde(rename="runtimeDir")]
  pub runtime_dir: Option<String>,

  pub generate_transitive: bool,

  pub include_resolver: bool,

  #[serde(default="TypescriptGenOptions::def_ts_style")]
  pub ts_style: TsStyle,

  pub modules: ModuleSrc,

  pub capitalize_branch_names_in_types: bool,

  pub capitalize_type_names: bool,
}

impl TypescriptGenOptions {
  pub fn new(outputs: OutputOpts, include_runtime: bool, runtime_dir: Option<String>, generate_transitive: bool, include_resolver: bool, modules: ModuleSrc, capitalize_branch_names_in_types: bool, capitalize_type_names: bool) -> TypescriptGenOptions {
    TypescriptGenOptions {
      referenceable: TypescriptGenOptions::def_referenceable(),
      outputs: outputs,
      include_runtime: include_runtime,
      runtime_dir: runtime_dir,
      generate_transitive: generate_transitive,
      include_resolver: include_resolver,
      ts_style: TypescriptGenOptions::def_ts_style(),
      modules: modules,
      capitalize_branch_names_in_types: capitalize_branch_names_in_types,
      capitalize_type_names: capitalize_type_names,
    }
  }

  pub fn def_referenceable() -> ReferenceableScopeOption {
    ReferenceableScopeOption::Local
  }

  pub fn def_ts_style() -> TsStyle {
    TsStyle::Tsc
  }
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct OutputOpts {
  #[serde(rename="outputDir")]
  pub output_dir: String,

  #[serde(default="OutputOpts::def_manifest")]
  pub manifest: Option<String>,
}

impl OutputOpts {
  pub fn new(output_dir: String) -> OutputOpts {
    OutputOpts {
      output_dir: output_dir,
      manifest: OutputOpts::def_manifest(),
    }
  }

  pub fn def_manifest() -> Option<String> {
    None
  }
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub enum TsStyle {
  #[serde(rename="tsc")]
  Tsc,

  #[serde(rename="deno")]
  Deno,
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub enum ModuleSrc {
  #[serde(rename="all")]
  All,

  #[serde(rename="modules")]
  Modules(Vec<String>),
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub enum ReferenceableScopeOption {
  /**
   * Generated code will only be referred internal to the repo
   */
  #[serde(rename="local")]
  Local,

  /**
   * Generated code can be published via a package manager (e.g. npm)
   */
  #[serde(rename="remote")]
  Remote,
}

/**
 * Expected to live in a file named `adl.pkg.json`
 */
#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct AdlPackage {
  pub pkg: PackageDirective,

  /**
   * Version
   */
  pub adlc: String,

  #[serde(default="AdlPackage::def_requires")]
  pub requires: Vec<Require>,

  #[serde(default="AdlPackage::def_excludes")]
  pub excludes: Vec<Exclude>,

  #[serde(default="AdlPackage::def_replaces")]
  pub replaces: Vec<Replace>,

  #[serde(default="AdlPackage::def_retracts")]
  pub retracts: Vec<Retract>,
}

impl AdlPackage {
  pub fn new(pkg: PackageDirective, adlc: String) -> AdlPackage {
    AdlPackage {
      pkg: pkg,
      adlc: adlc,
      requires: AdlPackage::def_requires(),
      excludes: AdlPackage::def_excludes(),
      replaces: AdlPackage::def_replaces(),
      retracts: AdlPackage::def_retracts(),
    }
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
pub struct PackageDirective {
  pub path: String,

  #[serde(default="PackageDirective::def_repo")]
  pub repo: Option<String>,
}

impl PackageDirective {
  pub fn new(path: String) -> PackageDirective {
    PackageDirective {
      path: path,
      repo: PackageDirective::def_repo(),
    }
  }

  pub fn def_repo() -> Option<String> {
    None
  }
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct Require {
  pub path: String,

  pub version: String,

  #[serde(default="Require::def_indirect")]
  pub indirect: bool,
}

impl Require {
  pub fn new(path: String, version: String) -> Require {
    Require {
      path: path,
      version: version,
      indirect: Require::def_indirect(),
    }
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
