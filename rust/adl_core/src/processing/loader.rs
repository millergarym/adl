use anyhow::anyhow;
use nom::Finish;
use nom_locate::LocatedSpan;
use std::collections::hash_map::Entry;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use crate::adlgen::adlc::bundle::AdlBundle;
use crate::adlgen::adlc::workspace::{
    EmbeddedBundle, InjectAnnotation, InjectAnnotations, LoaderRefType, LoaderWorkspace,
};
use crate::adlgen::sys::adlast2::{self as adlast, Module0};
use crate::parser::{convert_error, raw_module};
use crate::processing::annotations::apply_explicit_annotations_and_serialized_name;

use serde::Deserialize;

pub fn loader_from_workspace(root: PathBuf, workspace: LoaderWorkspace) -> Box<dyn AdlLoader> {
    Box::new(WorkspaceLoader {
        root,
        workspace,
        loaders: HashMap::new(),
    })
}

pub fn loader_from_search_paths(paths: &Vec<PathBuf>) -> Result<Box<dyn AdlLoader>, anyhow::Error> {
    let l_e: Result<Vec<Box<dyn AdlLoader>>, anyhow::Error> = paths.into_iter().map(loader_from_dir_tree).collect();
    match l_e {
        Ok(loaders) => Ok(Box::new(MultiLoader::new(loaders))),
        Err(e) => Err(e),
    }
}

pub fn loader_from_dir_tree(path: &PathBuf) -> Result<Box<dyn AdlLoader>, anyhow::Error> {
    let p_path = path.join("adl.bundle.json");
    let content = fs::read_to_string(&p_path).map_err(|e| anyhow!("Can't read pkg specified in workspace.\n\t package {:?}\n\t error: {}", p_path, e.to_string()))?;
    // let mut de = serde_json::Deserializer::from_str(&content);
    let de = &mut serde_json::Deserializer::from_str(&content);

    // let pkg = AdlBundle::deserialize(&mut de)
    //     .map_err(|e| anyhow!("{:?}: {}", p_path, e.to_string()))?;

    let mut unused = BTreeSet::new();
    let pkg: AdlBundle = serde_ignored::deserialize(de, |path| {
        unused.insert(path.to_string());
    })
    .map_err(|e| anyhow!("{:?}: {}", p_path, e.to_string()))?;
    if unused.len() != 0 {
        return Err(anyhow!("unknown fields `{:?}` {:?}", unused, p_path));
    }

    Ok(Box::new(DirTreeLoader::new(path.clone(), pkg)))
}

pub trait AdlLoader {
    /// Find and load the specified ADL module
    fn load(
        &mut self,
        module_name: &adlast::ModuleName,
    ) -> Result<Option<(Module0, AdlBundle, Option<InjectAnnotations>)>, anyhow::Error>;
    fn debug(&self);
}

pub struct WorkspaceLoader {
    root: PathBuf,
    workspace: LoaderWorkspace,
    loaders: HashMap<String, Box<dyn AdlLoader>>,
}

impl AdlLoader for WorkspaceLoader {
    fn debug(&self) {
        println!("!!!! WorkspaceLoader {:?} {:?}", self.root, self.workspace);
        let keys: &Vec<&String> = &self.loaders.keys().collect();
        // let &mut keys: &mut Vec<&String> = &self.loaders.keys().collect();
        // keys.sort();
        for k in keys.iter() {
            self.loaders.get(k.clone()).unwrap().debug();
        }
    }

    fn load(
        &mut self,
        module_name: &adlast::ModuleName,
    ) -> Result<Option<(Module0, AdlBundle, Option<InjectAnnotations>)>, anyhow::Error> {
        // if let Some(embedded_sys_loader_ref) = &self.workspace.embedded_sys_loader.0 {
        //     if let Some(mut module) = self.embedded.load(module_name)? {
        //         if module_name != "sys.annotations" {
        //             for an in &embedded_sys_loader_ref.loader_inject_annotate {
        //                 match an {
        //                     InjectAnnotation::Module(man) => {
        //                         module
        //                             .0
        //                             .annotations
        //                             .0
        //                             .insert(man.0 .0.clone(), man.0 .1.clone());
        //                     }
        //                 }
        //             }
        //         }
        //         module.1 = Some(embedded_sys_loader_ref.resolver_inject_annotate.clone());
        //         return Ok(Some(module));
        //     }
        // }
        for pkg in &self.workspace.r#use {
            let loader = match pkg.r#ref.clone() {
                LoaderRefType::Dir(d) => {
                    let pkg_path = d.bundle.as_str();
                    // println!("  looking for {} in {} or {:?}", module_name, pkg_path, pkg.0.1.global_alias.clone());
                    let pkg_name = if module_name.starts_with(pkg_path) {
                        Some(d.bundle.clone())
                    } else if let Some(alias) = d.module_prefix.clone() {
                        if module_name.starts_with(alias.as_str()) {
                            Some(alias)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    if pkg_name == None {
                        continue;
                    }
                    let name = pkg_name.unwrap();
                    match self.loaders.entry(name.clone()) {
                        Entry::Occupied(entry) => entry.into_mut(),
                        Entry::Vacant(entry) => {
                            let l = loader_from_dir_tree(&self.root.join(&d.path))?;
                            entry.insert(l)
                        },
                    }
                    // let l_e = self.loaders.entry(name.clone());
                    // match l_e {
                    //     Entry::Occupied(loader) => {
                    //         let l_exist = loader.into_mut();
                    //         l_exist
                    //     },
                    //     Entry::Vacant(entry) => {
                    //         let p2 = self.root.join(&d.path);
                    //         let l_new = &mut loader_from_dir_tree(&p2 )?;
                    //         entry.insert(*l_new);
                    //         // self.loaders.insert(name.clone(), l_new);
                    //         l_new
                    //     },
                    // }
                    // let l_es = self
                    //     .loaders
                    //     .entry(name.clone())
                    //     .or_insert(loader_from_dir_tree(self.root.join(&d.path)))
                    //     ;
                    // loader
                }
                LoaderRefType::Embedded(e) => self
                    .loaders
                    .entry(format!("{:?}", e))
                    .or_insert(Box::new(EmbeddedStdlibLoader { pkg: e })),
            };
            let module0 = loader.load(module_name);
            match module0 {
                Ok(module2) => {
                    if let Some((mut module1, _, _)) = module2.clone() {
                        for an in &pkg.loader_inject_annotate {
                            match an {
                                InjectAnnotation::Module(man) => {
                                    if *module_name != "sys.annotations".to_string() {
                                        if man.0 .0.module_name == *module_name {
                                            module1.annotations.0.insert(
                                                adlast::ScopedName {
                                                    module_name: "".to_string(),
                                                    name: man.0 .0.name.clone(),
                                                },
                                                man.0 .1.clone(),
                                            );
                                        } else {
                                            module1
                                                .annotations
                                                .0
                                                .insert(man.0 .0.clone(), man.0 .1.clone());
                                        }
                                    }
                                }
                            }
                        }
                        return Ok(Some((
                            module1,
                            pkg.bundle.clone(),
                            Some(pkg.resolver_inject_annotate.clone()),
                        )));
                    } else {
                        // return Ok(None);
                        // return Err(anyhow!("Module not found '{}'", module_name));
                    }
                }
                Err(_) => return module0,
            }
        }
        Ok(None)
    }
}

/// Combines a bunch of loaders
pub struct MultiLoader {
    loaders: Vec<Box<dyn AdlLoader>>,
}

impl MultiLoader {
    pub fn new(loaders: Vec<Box<dyn AdlLoader>>) -> Self {
        MultiLoader { loaders }
    }
}

impl AdlLoader for MultiLoader {
    fn debug(&self) {
        println!("!!!! MultiLoader");
        for l in &self.loaders {
            l.debug();
        }
    }

    fn load(
        &mut self,
        module_name: &adlast::ModuleName,
    ) -> Result<Option<(Module0, AdlBundle, Option<InjectAnnotations>)>, anyhow::Error> {
        for loader in &mut self.loaders {
            if let Some(module) = loader.load(module_name)? {
                return Ok(Some(module));
            }
        }
        Ok(None)
    }
}

pub struct EmbeddedStdlibLoader {
    pkg: EmbeddedBundle,
}

impl AdlLoader for EmbeddedStdlibLoader {
    fn debug(&self) {
        println!("!!! EmbeddedStdlibLoader {:?}", self.pkg);
    }

    fn load(
        &mut self,
        module_name: &adlast::ModuleName,
    ) -> Result<Option<(Module0, AdlBundle, Option<InjectAnnotations>)>, anyhow::Error> {
        if let Some((pkg, data)) = crate::adlstdlib::get_stdlib(&self.pkg, module_name, "") {
            match std::str::from_utf8(data.as_ref()) {
                Ok(content) => return parse(&content).map(|m| Some((m, pkg, None))),
                Err(err) => return Err(anyhow::Error::from(err)),
            }
        } else {
            Ok(None)
        }
    }
}

pub struct DirTreeLoader {
    root: PathBuf,
    bundle: AdlBundle,
}

impl DirTreeLoader {
    pub fn new(root: PathBuf, bundle: AdlBundle) -> Self {
        DirTreeLoader { root, bundle }
    }
}

impl AdlLoader for DirTreeLoader {
    fn debug(&self) {
        println!("!!!! DirTreeLoader {:?}", self.root);
    }

    fn load(
        &mut self,
        module_name: &adlast::ModuleName,
    ) -> Result<Option<(Module0, AdlBundle, Option<InjectAnnotations>)>, anyhow::Error> {
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
        parse(&content).map(|m| Some((m, self.bundle.clone(), None)))
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
