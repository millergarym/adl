// @generated from adl module adlc.packaging

use crate::adlgen::sys::adlast2::Module1;
use crate::adlgen::sys::adlast2::ScopedName;
use crate::adlrt::custom::sys::types::pair::Pair;
use serde::Deserialize;
use serde::Serialize;

pub type AdlWorkspace0 = AdlWorkspace<AdlPackageRef>;

pub type AdlWorkspace1 = AdlWorkspace<Payload1>;

pub type AdlWorkspace2 = AdlWorkspace<Payload2>;

#[derive(Clone,Debug,Deserialize,PartialEq,Serialize)]
pub struct Payload1 {
  pub p_ref: AdlPackageRef,

  pub pkg: AdlPackage,
}

impl Payload1 {
  pub fn new(p_ref: AdlPackageRef, pkg: AdlPackage) -> Payload1 {
    Payload1 {
      p_ref: p_ref,
      pkg: pkg,
    }
  }
}

#[derive(Clone,Debug,Deserialize,PartialEq,Serialize)]
pub struct Payload2 {
  pub p_ref: AdlPackageRef,

  pub pkg: AdlPackage,

  pub modules: Vec<Module1>,
}

impl Payload2 {
  pub fn new(p_ref: AdlPackageRef, pkg: AdlPackage, modules: Vec<Module1>) -> Payload2 {
    Payload2 {
      p_ref: p_ref,
      pkg: pkg,
      modules: modules,
    }
  }
}

/**
 * Expected to live in a file named `adl.work.json`
 */
#[derive(Clone,Debug,Deserialize,PartialEq,Serialize)]
pub struct AdlWorkspace<T> {
  pub adlc: String,

  #[serde(rename="use")]
  pub r#use: Vec<T>,

  #[serde(default="AdlWorkspace::<T>::def_runtimes")]
  pub runtimes: Vec<RuntimeOpts>,

  #[serde(default="AdlWorkspace::<T>::def_embedded_sys_loader")]
  pub embedded_sys_loader: Option<Payload1>,
}

impl<T> AdlWorkspace<T> {
  pub fn new(adlc: String, r#use: Vec<T>) -> AdlWorkspace<T> {
    AdlWorkspace {
      adlc: adlc,
      r#use: r#use,
      runtimes: AdlWorkspace::<T>::def_runtimes(),
      embedded_sys_loader: AdlWorkspace::<T>::def_embedded_sys_loader(),
    }
  }

  pub fn def_runtimes() -> Vec<RuntimeOpts> {
    vec![]
  }

  pub fn def_embedded_sys_loader() -> Option<Payload1> {
    Some(Payload1{p_ref : AdlPackageRef{path : "".to_string(), ts_opts : Some(TypescriptGenOptions{npm_pkg_name : "@adl-lang/sys".to_string(), npm_version : "1.0.0".to_string(), extra_dependencies : [("base64-js".to_string(), "^1.5.1".to_string())].iter().cloned().collect(), extra_dev_dependencies : [("tsconfig".to_string(), "workspace:*".to_string()), ("typescript".to_string(), "^4.9.3".to_string())].iter().cloned().collect(), outputs : None, runtime_opts : TsRuntimeOpt::PackageRef(NpmPackageRef{name : "@adl-lang/runtime".to_string(), version : "^1.0.0".to_string()}), generate_transitive : false, include_resolver : false, ts_style : TsStyle::Tsc, modules : ModuleSrc::All, capitalize_branch_names_in_types : true, capitalize_type_names : true, annotate : vec![]})}, pkg : AdlPackage{path : "github.com/adl-lang/adl/adl/stdlib/sys".to_string(), global_alias : Some("sys".to_string()), adlc : "0.0.0".to_string(), requires : vec![], excludes : vec![], replaces : vec![], retracts : vec![]}})
  }
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub enum RuntimeOpts {
  #[serde(rename="ts_runtime")]
  TsRuntime(TsWriteRuntime),
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct TsWriteRuntime {
  pub output_dir: String,

  #[serde(default="TsWriteRuntime::def_referenceable")]
  pub referenceable: ReferenceableScopeOption,

  #[serde(default="TsWriteRuntime::def_npm_pkg_name")]
  pub npm_pkg_name: String,

  #[serde(default="TsWriteRuntime::def_ts_style")]
  pub ts_style: TsStyle,

  #[serde(default="TsWriteRuntime::def_strip_first")]
  pub strip_first: bool,
}

impl TsWriteRuntime {
  pub fn new(output_dir: String) -> TsWriteRuntime {
    TsWriteRuntime {
      output_dir: output_dir,
      referenceable: TsWriteRuntime::def_referenceable(),
      npm_pkg_name: TsWriteRuntime::def_npm_pkg_name(),
      ts_style: TsWriteRuntime::def_ts_style(),
      strip_first: TsWriteRuntime::def_strip_first(),
    }
  }

  pub fn def_referenceable() -> ReferenceableScopeOption {
    ReferenceableScopeOption::Local
  }

  pub fn def_npm_pkg_name() -> String {
    "@adl-lang/runtime".to_string()
  }

  pub fn def_ts_style() -> TsStyle {
    TsStyle::Tsc
  }

  pub fn def_strip_first() -> bool {
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
  pub npm_pkg_name: String,

  #[serde(default="TypescriptGenOptions::def_npm_version")]
  pub npm_version: String,

  #[serde(default="TypescriptGenOptions::def_extra_dependencies")]
  pub extra_dependencies: std::collections::HashMap<String,VersionSpec>,

  #[serde(default="TypescriptGenOptions::def_extra_dev_dependencies")]
  pub extra_dev_dependencies: std::collections::HashMap<String,VersionSpec>,

  #[serde(default="TypescriptGenOptions::def_outputs")]
  pub outputs: Option<OutputOpts>,

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
  pub fn new(npm_pkg_name: String) -> TypescriptGenOptions {
    TypescriptGenOptions {
      npm_pkg_name: npm_pkg_name,
      npm_version: TypescriptGenOptions::def_npm_version(),
      extra_dependencies: TypescriptGenOptions::def_extra_dependencies(),
      extra_dev_dependencies: TypescriptGenOptions::def_extra_dev_dependencies(),
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

  pub fn def_npm_version() -> String {
    "1.0.0".to_string()
  }

  pub fn def_extra_dependencies() -> std::collections::HashMap<String,VersionSpec> {
    [].iter().cloned().collect()
  }

  pub fn def_extra_dev_dependencies() -> std::collections::HashMap<String,VersionSpec> {
    [].iter().cloned().collect()
  }

  pub fn def_outputs() -> Option<OutputOpts> {
    None
  }

  pub fn def_runtime_opts() -> TsRuntimeOpt {
    TsRuntimeOpt::PackageRef(NpmPackageRef{name : "@adl-lang/runtime".to_string(), version : "^1.0.0".to_string()})
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
  #[serde(rename="workspace_ref")]
  WorkspaceRef(String),

  #[serde(rename="package_ref")]
  PackageRef(NpmPackageRef),

  #[serde(rename="generate")]
  Generate(TsGenRuntime),
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct TsGenRuntime {
}

impl TsGenRuntime {
  pub fn new() -> TsGenRuntime {
    TsGenRuntime {
    }
  }
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub enum OutputOpts {
  #[serde(rename="gen")]
  Gen(GenOutput),
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct GenOutput {
  #[serde(default="GenOutput::def_referenceable")]
  pub referenceable: ReferenceableScopeOption,

  pub output_dir: String,

  #[serde(default="GenOutput::def_manifest")]
  pub manifest: Option<String>,

  /**
   * When creating the path for output ts files delete the first part of the module name
   * This needs to be false for "generate_transitive" and 
   * packages like "common" where the module and directory at the top level named the same.
   */
  #[serde(default="GenOutput::def_strip_first")]
  pub strip_first: bool,
}

impl GenOutput {
  pub fn new(output_dir: String) -> GenOutput {
    GenOutput {
      referenceable: GenOutput::def_referenceable(),
      output_dir: output_dir,
      manifest: GenOutput::def_manifest(),
      strip_first: GenOutput::def_strip_first(),
    }
  }

  pub fn def_referenceable() -> ReferenceableScopeOption {
    ReferenceableScopeOption::Local
  }

  pub fn def_manifest() -> Option<String> {
    None
  }

  pub fn def_strip_first() -> bool {
    true
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
  #[serde(rename="ref")]
  pub r#ref: PkgRef,

  #[serde(default="Require::def_version")]
  pub version: Option<String>,

  #[serde(default="Require::def_indirect")]
  pub indirect: bool,
}

impl Require {
  pub fn new(r#ref: PkgRef) -> Require {
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
pub enum PkgRef {
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

#[derive(Clone,Debug,Deserialize,Eq,PartialEq,Serialize)]
pub struct NpmPackage {
  pub name: String,

  pub version: String,

  #[serde(default="NpmPackage::def_scripts")]
  pub scripts: std::collections::HashMap<String,String>,

  #[serde(default="NpmPackage::def_dependencies")]
  pub dependencies: std::collections::HashMap<String,String>,

  #[serde(default="NpmPackage::def_dev_dependencies")]
  #[serde(rename="devDependencies")]
  pub dev_dependencies: std::collections::HashMap<String,String>,
}

impl NpmPackage {
  pub fn new(name: String, version: String) -> NpmPackage {
    NpmPackage {
      name: name,
      version: version,
      scripts: NpmPackage::def_scripts(),
      dependencies: NpmPackage::def_dependencies(),
      dev_dependencies: NpmPackage::def_dev_dependencies(),
    }
  }

  pub fn def_scripts() -> std::collections::HashMap<String,String> {
    [].iter().cloned().collect()
  }

  pub fn def_dependencies() -> std::collections::HashMap<String,String> {
    [].iter().cloned().collect()
  }

  pub fn def_dev_dependencies() -> std::collections::HashMap<String,String> {
    [].iter().cloned().collect()
  }
}

#[derive(Clone,Debug,Deserialize,Eq,Hash,PartialEq,Serialize)]
pub struct NpmPackageRef {
  pub name: String,

  pub version: VersionSpec,
}

impl NpmPackageRef {
  pub fn new(name: String, version: VersionSpec) -> NpmPackageRef {
    NpmPackageRef {
      name: name,
      version: version,
    }
  }
}

pub type VersionSpec = String;
