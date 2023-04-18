// @generated from adl module adlc.packaging

use crate::adlgen::sys::adlast2::ScopedName;
use crate::adlrt::custom::sys::types::pair::Pair;
use serde::Deserialize;
use serde::Serialize;

pub type AdlWorkspace0 = AdlWorkspace<AdlPackageRef>;

pub type AdlWorkspace1 = AdlWorkspace<Pair<AdlPackageRef, AdlPackage>>;

/**
 * Expected to live in a file named `adl.work.json`
 */
#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct AdlWorkspace<T> {
  pub adlc: String,

  #[serde(rename="use")]
  pub r#use: Vec<T>,

  #[serde(default="AdlWorkspace::<T>::def_use_embedded_sys_loader")]
  pub use_embedded_sys_loader: bool,
}

impl<T> AdlWorkspace<T> {
  pub fn new(adlc: String, r#use: Vec<T>) -> AdlWorkspace<T> {
    AdlWorkspace {
      adlc: adlc,
      r#use: r#use,
      use_embedded_sys_loader: AdlWorkspace::<T>::def_use_embedded_sys_loader(),
    }
  }

  pub fn def_use_embedded_sys_loader() -> bool {
    true
  }
}

pub type AdlPackageRefs = Vec<AdlPackageRef>;

#[derive(Clone,Debug,Deserialize,PartialEq,Serialize)]
pub struct AdlPackageRef {
  pub path: String,

  #[serde(default="AdlPackageRef::def_ts_opts")]
  pub ts_opts: Option<TypescriptGenOptions>,
}

impl AdlPackageRef {
  pub fn new(path: String) -> AdlPackageRef {
    AdlPackageRef {
      path: path,
      ts_opts: AdlPackageRef::def_ts_opts(),
    }
  }

  pub fn def_ts_opts() -> Option<TypescriptGenOptions> {
    None
  }
}

#[derive(Clone,Debug,Deserialize,PartialEq,Serialize)]
pub struct TypescriptGenOptions {
  pub npm_pkg_name: Option<String>,

  #[serde(default="TypescriptGenOptions::def_outputs")]
  pub outputs: OutputOpts,

  #[serde(default="TypescriptGenOptions::def_runtime_opts")]
  pub runtime_opts: TsRuntimeOpt,

  #[serde(default="TypescriptGenOptions::def_generate_transitive")]
  pub generate_transitive: bool,

  #[serde(default="TypescriptGenOptions::def_include_resolver")]
  pub include_resolver: bool,

  #[serde(default="TypescriptGenOptions::def_ts_style")]
  pub ts_style: TsStyle,

  #[serde(default="TypescriptGenOptions::def_modules")]
  pub modules: ModuleSrc,

  #[serde(default="TypescriptGenOptions::def_capitalize_branch_names_in_types")]
  pub capitalize_branch_names_in_types: bool,

  #[serde(default="TypescriptGenOptions::def_capitalize_type_names")]
  pub capitalize_type_names: bool,

  #[serde(default="TypescriptGenOptions::def_annotate")]
  pub annotate: Vec<InjectAnnotation>,
}

impl TypescriptGenOptions {
  pub fn new(npm_pkg_name: Option<String>) -> TypescriptGenOptions {
    TypescriptGenOptions {
      npm_pkg_name: npm_pkg_name,
      outputs: TypescriptGenOptions::def_outputs(),
      runtime_opts: TypescriptGenOptions::def_runtime_opts(),
      generate_transitive: TypescriptGenOptions::def_generate_transitive(),
      include_resolver: TypescriptGenOptions::def_include_resolver(),
      ts_style: TypescriptGenOptions::def_ts_style(),
      modules: TypescriptGenOptions::def_modules(),
      capitalize_branch_names_in_types: TypescriptGenOptions::def_capitalize_branch_names_in_types(),
      capitalize_type_names: TypescriptGenOptions::def_capitalize_type_names(),
      annotate: TypescriptGenOptions::def_annotate(),
    }
  }

  pub fn def_outputs() -> OutputOpts {
    OutputOpts::Ref(PkgRef{})
  }

  pub fn def_runtime_opts() -> TsRuntimeOpt {
    TsRuntimeOpt::PackageRef("@adl-lang/runtime".to_string())
  }

  pub fn def_generate_transitive() -> bool {
    false
  }

  pub fn def_include_resolver() -> bool {
    false
  }

  pub fn def_ts_style() -> TsStyle {
    TsStyle::Tsc
  }

  pub fn def_modules() -> ModuleSrc {
    ModuleSrc::All
  }

  pub fn def_capitalize_branch_names_in_types() -> bool {
    true
  }

  pub fn def_capitalize_type_names() -> bool {
    true
  }

  pub fn def_annotate() -> Vec<InjectAnnotation> {
    vec![]
  }
}

#[derive(Clone,Debug,Deserialize,PartialEq,Serialize)]
pub enum InjectAnnotation {
  #[serde(rename="module_")]
  Module(Pair<ScopedName, serde_json::Value>),
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub enum TsRuntimeOpt {
  #[serde(rename="packageRef")]
  PackageRef(String),

  #[serde(rename="generate")]
  Generate(TsGenRuntime),
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct TsGenRuntime {
  #[serde(rename="runtimeDir")]
  pub runtime_dir: String,
}

impl TsGenRuntime {
  pub fn new(runtime_dir: String) -> TsGenRuntime {
    TsGenRuntime {
      runtime_dir: runtime_dir,
    }
  }
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub enum OutputOpts {
  #[serde(rename="gen")]
  Gen(GenOutput),

  #[serde(rename="ref")]
  Ref(PkgRef),
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct PkgRef {
}

impl PkgRef {
  pub fn new() -> PkgRef {
    PkgRef {
    }
  }
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct GenOutput {
  #[serde(default="GenOutput::def_referenceable")]
  pub referenceable: ReferenceableScopeOption,

  #[serde(rename="outputDir")]
  pub output_dir: String,

  #[serde(default="GenOutput::def_manifest")]
  pub manifest: Option<String>,
}

impl GenOutput {
  pub fn new(output_dir: String) -> GenOutput {
    GenOutput {
      referenceable: GenOutput::def_referenceable(),
      output_dir: output_dir,
      manifest: GenOutput::def_manifest(),
    }
  }

  pub fn def_referenceable() -> ReferenceableScopeOption {
    ReferenceableScopeOption::Local
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
  pub path: String,

  #[serde(default="AdlPackage::def_global_alias")]
  #[serde(rename="globalAlias")]
  pub global_alias: Option<String>,

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
  pub fn new(path: String, adlc: String) -> AdlPackage {
    AdlPackage {
      path: path,
      global_alias: AdlPackage::def_global_alias(),
      adlc: adlc,
      requires: AdlPackage::def_requires(),
      excludes: AdlPackage::def_excludes(),
      replaces: AdlPackage::def_replaces(),
      retracts: AdlPackage::def_retracts(),
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
