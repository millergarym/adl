use anyhow::anyhow;
use nom::Finish;
use nom_locate::LocatedSpan;
use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use crate::adlgen::adlc::packaging::{AdlPackage, AdlWorkspace1};
use crate::adlgen::sys::adlast2::{self as adlast, Module0};
use crate::parser::{convert_error, raw_module};
use crate::processing::annotations::apply_explicit_annotations_and_serialized_name;

pub fn loader_from_workspace(
    root: PathBuf,
    workspace: AdlWorkspace1,
) -> Box<dyn AdlLoader> {
    Box::new(WorkspaceLoader {
        root,
        workspace,
        embedded: EmbeddedStdlibLoader {},
        loaders: HashMap::new(),
    })
}

pub fn loader_from_search_paths(paths: &Vec<PathBuf>) -> Box<dyn AdlLoader> {
    let loaders = paths
        .iter()
        .map(|r| loader_from_dir_tree(r, None))
        .collect();
    Box::new(MultiLoader::new(loaders))
}

pub fn loader_from_dir_tree(root: &PathBuf, adl_pkg: Option<AdlPackage>) -> Box<dyn AdlLoader> {
    Box::new(DirTreeLoader::new(root.clone(), adl_pkg))
}

pub trait AdlLoader {
    /// Find and load the specified ADL module
    fn load(
        &mut self,
        module_name: &adlast::ModuleName,
    ) -> Result<Option<(Option<AdlPackage>, Module0)>, anyhow::Error>;
}

pub struct WorkspaceLoader {
    root: PathBuf,
    workspace: AdlWorkspace1,
    embedded: EmbeddedStdlibLoader,
    loaders: HashMap<String, Box<dyn AdlLoader>>,
}

impl AdlLoader for WorkspaceLoader {
    fn load(
        &mut self,
        module_name: &adlast::ModuleName,
    ) -> Result<Option<(Option<AdlPackage>, Module0)>, anyhow::Error> {
        if self.workspace.use_embedded_sys_loader {
            if let Some(module) = self.embedded.load(module_name)? {
                return Ok(Some(module));
            }
        }
        for pkg in &self.workspace.r#use {
            let pkg_path = pkg.0 .1.path.as_str();
            println!(
                "  looking for {} in {} or {:?}",
                module_name,
                pkg_path,
                pkg.0 .1.global_alias.clone()
            );
            let pkg_name = if module_name.starts_with(pkg_path) {
                Some(pkg.0 .1.path.clone())
            } else if let Some(alias) = pkg.0 .1.global_alias.clone() {
                if module_name.starts_with(alias.as_str()) {
                    Some(alias)
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(name) = pkg_name {
                let loader = self
                    .loaders
                    .entry(name)
                    .or_insert(Box::new(DirTreeLoader::new(
                        self.root.join(&pkg.0 .0.path),
                        Some(pkg.0 .1.clone()),
                    )));
                return loader.load(module_name);
            }
        }
        Ok(None)
    }
}

/// Combines a bunch of loaders
pub struct MultiLoader {
    embedded: EmbeddedStdlibLoader,
    loaders: Vec<Box<dyn AdlLoader>>,
}

impl MultiLoader {
    pub fn new(loaders: Vec<Box<dyn AdlLoader>>) -> Self {
        MultiLoader {
            embedded: EmbeddedStdlibLoader {},
            loaders,
        }
    }
}

impl AdlLoader for MultiLoader {
    fn load(
        &mut self,
        module_name: &adlast::ModuleName,
    ) -> Result<Option<(Option<AdlPackage>, Module0)>, anyhow::Error> {
        if let Some(module) = self.embedded.load(module_name)? {
            return Ok(Some(module));
        }
        for loader in &mut self.loaders {
            if let Some(module) = loader.load(module_name)? {
                return Ok(Some(module));
            }
        }
        Ok(None)
    }
}

pub struct EmbeddedStdlibLoader {}

fn adl_runtime_pkg() -> AdlPackage {
    AdlPackage {
        path: "github.com/adl-lang/adl/adl/stdlib/sys".to_string(),
        global_alias: Some(String::from("sys")),
        adlc: String::from("0.0.0"),
        requires: vec![],
        excludes: vec![],
        replaces: vec![],
        retracts: vec![],
    }
}

impl AdlLoader for EmbeddedStdlibLoader {
    fn load(
        &mut self,
        module_name: &adlast::ModuleName,
    ) -> Result<Option<(Option<AdlPackage>, Module0)>, anyhow::Error> {
        match crate::adlstdlib::get_stdlib(module_name, "") {
            Some(data) => match std::str::from_utf8(data.as_ref()) {
                Ok(content) => return parse(&content).map(|m| Some((Some(adl_runtime_pkg()), m))),
                Err(err) => return Err(anyhow::Error::from(err)),
            },
            None => return Ok(None),
        }
    }
}

pub struct DirTreeLoader {
    root: PathBuf,
    adl_pkg: Option<AdlPackage>,
}

impl DirTreeLoader {
    pub fn new(root: PathBuf, adl_pkg: Option<AdlPackage>) -> Self {
        DirTreeLoader { root, adl_pkg }
    }
}

impl AdlLoader for DirTreeLoader {
    fn load(
        &mut self,
        module_name: &adlast::ModuleName,
    ) -> Result<Option<(Option<AdlPackage>, Module0)>, anyhow::Error> {
        let mut path = self.root.clone();
        for mp in module_name.split(".") {
            path.push(mp);
        }
        path.set_extension("adl");
        let econtent = fs::read_to_string(path.clone());
        let content = match econtent {
            Err(err) => match err.kind() {
                ErrorKind::NotFound => return Ok(None),
                _ => return Err(anyhow::Error::from(err)),
            },
            Ok(content) => content,
        };
        log::info!("loaded {} from {}", module_name, path.display());
        parse(&content).map(|m| Some((self.adl_pkg.clone(), m)))
    }
}

fn parse(content: &str) -> Result<Module0, anyhow::Error> {
    let inp = LocatedSpan::new(content);
    let (_, raw_module) = raw_module(inp)
        .finish()
        .map_err(|e| anyhow!(convert_error(inp, e)))?;
    match apply_explicit_annotations_and_serialized_name(raw_module) {
        Ok(module0) => Ok(module0),
        Err(err) => Err(anyhow::Error::from(err)),
    }
}
