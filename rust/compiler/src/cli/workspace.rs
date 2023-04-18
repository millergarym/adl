use std::path::{Path, PathBuf};
use std::{env, fs};

use anyhow::anyhow;

use serde::Deserialize;

use crate::adlgen::adlc::packaging::{AdlPackage, AdlWorkspace0, AdlWorkspace1, OutputOpts, TsRuntimeOpt};
use crate::adlrt::custom::sys::types::pair::Pair;
use crate::processing::loader::loader_from_workspace;

use super::TsOpts;
use super::{tsgen, TsStyle};

pub(crate) fn workspace(opts: &super::GenOpts) -> Result<(), anyhow::Error> {
    let pkg_defs = collect_work_and_pkg(&opts.dir)?;
    let wrk1 = collection_to_workspace(pkg_defs)?;
    // println!("{:?}", &wrk1);
    for pkg in &wrk1.1.r#use {
        if let Some(opts) = &pkg.0.0.ts_opts {
            let outputs = match &opts.outputs {
                OutputOpts::Gen(output) => output,
                OutputOpts::Ref(_) => continue,
            };
            let rt = match &opts.runtime_opts {
                TsRuntimeOpt::PackageRef(rt) => (false, None, Some(rt.clone())),
                TsRuntimeOpt::Generate(rt) => (true, Some(rt.runtime_dir.clone()), None),
            };
            let tsopts = TsOpts {
                search: super::AdlSearchOpts { path: vec![] },
                output: super::OutputOpts {
                    outputdir: wrk1.0.join(outputs.output_dir.clone()),
                    manifest: outputs.manifest.clone().map(|m| wrk1.0.join(m)),
                },
                include_rt: rt.0,
                runtime_dir: rt.1,
                runtime_pkg: rt.2,
                generate_transitive: opts.generate_transitive,
                include_resolver: opts.include_resolver,
                ts_style: match opts.ts_style {
                    crate::adlgen::adlc::packaging::TsStyle::Tsc => Some(TsStyle::Tsc),
                    crate::adlgen::adlc::packaging::TsStyle::Deno => Some(TsStyle::Deno),
                },
                modules: match &opts.modules {
                    crate::adlgen::adlc::packaging::ModuleSrc::All => {
                        let pkg_root = wrk1.0.join(pkg.0 .0.path.clone()).canonicalize()?;
                        if let Some(pkg_root_str) = pkg_root.as_os_str().to_str() {
                            walk_and_collect_adl_modules(pkg_root_str, &pkg_root)
                        } else {
                            return Err(anyhow!("Could get str from pkg_root"));
                        }
                    }
                    crate::adlgen::adlc::packaging::ModuleSrc::Modules(ms) => ms.clone(),
                },
                capitalize_branch_names_in_types: opts.capitalize_branch_names_in_types,
                capitalize_type_names: opts.capitalize_type_names,
            };
            let loader = loader_from_workspace(wrk1.0.clone(), wrk1.1.clone());
            println!(
                "TsGen for pkg {:?} in workspace {:?} output dir {}",
                pkg.0 .0, wrk1.0, outputs.output_dir
            );
            tsgen::tsgen(loader, &tsopts)?;
        }
    }
    Ok(())
}

fn walk_and_collect_adl_modules(pkg_root: &str, cwd: &PathBuf) -> Vec<String> {
    let mut mods = vec![];
    if let Ok(files) = fs::read_dir(cwd) {
        for file in files {
            if let Ok(file) = file {
                let path = file.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "adl" {
                            if let Some(name) = path.to_str() {
                                let name1 = &name[(pkg_root.len() + 1)..(name.len() - 4)];
                                let name2 = name1.replace("/", ".");
                                println!("  adding module {}", name2);
                                mods.push(name2);
                            }
                        }
                    }
                }
                if path.is_dir() {
                    mods.append(&mut walk_and_collect_adl_modules(pkg_root, &path));
                }
            }
        }
    }
    mods
}

fn collection_to_workspace(
    pkg_defs: Vec<(PkgDef, PathBuf, &str)>,
) -> Result<(PathBuf, AdlWorkspace1), anyhow::Error> {
    for porw in pkg_defs {
        let porw_path = porw.1.join(porw.2);
        let content = fs::read_to_string(&porw_path).map_err(|e| anyhow!("{:?}: {}", porw_path, e.to_string()))?;
        let mut de = serde_json::Deserializer::from_str(&content);
        match porw.0 {
            PkgDef::Pkg => {
                // let pkg = AdlPackage::deserialize(&mut de).map_err(|e| anyhow!("{:?}: {}", porw_path, e.to_string()))?;
                // println!("pkg {:?}", pkg);
            }
            PkgDef::Work => {
                let wrk0 = AdlWorkspace0::deserialize(&mut de)
                    .map_err(|e| anyhow!("{:?}: {}", porw_path, e.to_string()))?;
                let mut wrk1 = AdlWorkspace1 {
                    adlc: wrk0.adlc.clone(),
                    r#use: vec![],
                    use_embedded_sys_loader: wrk0.use_embedded_sys_loader,
                };
                for p in wrk0.r#use.iter() {
                    let p_path = porw.1.join(&p.path).join("adl.pkg.json");
                    let content = fs::read_to_string(&p_path).map_err(|e| anyhow!("Can't read pkg specified in workspace.\n\tworkspace {:?}\n\t package {:?}\n\t error: {}", porw_path, p_path, e.to_string()))?;
                    let mut de = serde_json::Deserializer::from_str(&content);
                    let pkg = AdlPackage::deserialize(&mut de)
                        .map_err(|e| anyhow!("{:?}: {}", p_path, e.to_string()))?;
                    wrk1.r#use.push(Pair::new(p.clone(), pkg));
                }
                // println!("wrk {:?}", wrk1);
                return Ok((porw.1, wrk1));
            }
        }
    }
    Err(anyhow!("No workspace found"))
}

const ADL_PKG_FILES: &[(&str, PkgDef)] = &[
    ("adl.pkg.json", PkgDef::Pkg),
    ("adl.work.json", PkgDef::Work),
];

#[derive(Debug, Copy, Clone, PartialEq)]
enum PkgDef {
    Pkg,
    Work,
}

fn collect_work_and_pkg(start_dir: &String) -> Result<Vec<(PkgDef, PathBuf, &str)>, anyhow::Error> {
    let mut res = vec![];
    let current_dir = env::current_dir()?;
    let current_dir = current_dir.join(start_dir);
    let mut current_dir = current_dir.canonicalize()?;

    loop {
        for f in ADL_PKG_FILES {
            let file_path = current_dir.join(f.0);
            if file_path.exists() {
                res.push((f.1, current_dir.clone(), f.0));
            }
        }
        if !current_dir.pop() {
            break;
        }
    }
    Ok(res)
}
