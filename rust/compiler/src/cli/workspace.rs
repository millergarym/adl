use std::path::PathBuf;
use std::{env, fs};

use anyhow::anyhow;

use serde::Deserialize;

use crate::adlgen::adlc::packaging::{
    AdlPackage, AdlWorkspace0, AdlWorkspace1, InjectAnnotation, LoaderRef, LoaderWorkspace,
    Payload1,
};
use crate::adlgen::sys::adlast2::ScopedName;
use crate::adlrt::custom::sys::types::maybe::Maybe;
use crate::adlrt::custom::sys::types::pair::Pair;
use crate::processing::loader::loader_from_workspace;

use super::tsgen;

pub(crate) fn workspace(opts: &super::GenOpts) -> Result<(), anyhow::Error> {
    let pkg_defs = collect_work_and_pkg(opts)?;
    let wrk1 = collection_to_workspace(pkg_defs)?;
    // println!("{:?}", &wrk1);
    for pkg in &wrk1.1.r#use {
        let loader = loader_from_workspace(wrk1.0.clone(), wrk1_to_wld(wrk1.1.clone()));
        if let Some(opts) = &pkg.p_ref.ts_opts {
            println!(
                "TsGen for pkg {:?} in workspace {:?} output dir {:?}",
                pkg.p_ref, wrk1.0, &opts.outputs
            );
            let pkg_root = wrk1.0.join(pkg.p_ref.path.clone()).canonicalize()?;
            std::env::set_current_dir(&wrk1.0)?;
            tsgen::tsgen(loader, &opts, Some(pkg_root))?;
            tsgen::gen_npm_package(pkg.p_ref.path.clone(), &wrk1.1)?;
        }
    }
    for rt in &wrk1.1.runtimes {
        match rt {
            crate::adlgen::adlc::packaging::RuntimeOpts::TsRuntime(rt_opts) => {
                tsgen::write_runtime(rt_opts)?
            }
        }
    }
    Ok(())
}

fn wrk1_to_wld(wrk1: AdlWorkspace1) -> LoaderWorkspace {
    LoaderWorkspace {
        adlc: wrk1.adlc,
        r#use: wrk1
            .r#use
            .iter()
            .map(move |p| payload1_to_loader_ref(p))
            .collect(),
        runtimes: wrk1.runtimes,
        embedded_sys_loader: Maybe(wrk1.embedded_sys_loader.0.as_ref().map(payload1_to_loader_ref)),
    }
}

fn payload1_to_loader_ref(payload1: &Payload1) -> LoaderRef {
    let mut loader_ref = LoaderRef {
        path: payload1.p_ref.path.clone(),
        global_alias: payload1.pkg.global_alias.clone(),
        loader_inject_annotate: vec![],
        resolver_inject_annotate: vec![],
    };

    if let Some(ts_opts) = &payload1.p_ref.ts_opts {
        loader_ref
            .resolver_inject_annotate
            .push(InjectAnnotation::Module(Pair((
                ScopedName {
                    module_name: "adlc.config.typescript".to_string(),
                    name: "NpmPackage".to_string(),
                },
                serde_json::json!(&ts_opts.npm_pkg_name),
            ))));
        // let mn1 = "adlc.config.typescript".to_string();
        // module1.annotations.0.insert(
        //     adlast::ScopedName {
        //         module_name: if *module_name == mn1 {
        //             "".to_string()
        //         } else {
        //             mn1
        //         },
        //         name: "NpmPackage".to_string(),
        //     },
        //     serde_json::json!(&ts_opts.npm_pkg_name),
        // );
    }

    loader_ref
}

fn collection_to_workspace(
    pkg_defs: Vec<(PkgDef, PathBuf, &str)>,
) -> Result<(PathBuf, AdlWorkspace1), anyhow::Error> {
    for porw in pkg_defs {
        let porw_path = porw.1.join(porw.2);
        let content = fs::read_to_string(&porw_path)
            .map_err(|e| anyhow!("{:?}: {}", porw_path, e.to_string()))?;
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
                    runtimes: wrk0.runtimes,
                    r#use: vec![],
                    embedded_sys_loader: Maybe(wrk0.embedded_sys_loader.0.map(|esl| Payload1 {
                        p_ref: esl,
                        pkg: AdlPackage {
                            path: "github.com/adl-lang/adl/adl/stdlib/sys".to_string(),
                            global_alias: Some("sys".to_string()),
                            adlc: "0.0.0".to_string(),
                            requires: vec![],
                            excludes: vec![],
                            replaces: vec![],
                            retracts: vec![],
                        },
                    })),
                };
                for p in wrk0.r#use.iter() {
                    let p_path = porw.1.join(&p.path).join("adl.pkg.json");
                    let content = fs::read_to_string(&p_path).map_err(|e| anyhow!("Can't read pkg specified in workspace.\n\tworkspace {:?}\n\t package {:?}\n\t error: {}", porw_path, p_path, e.to_string()))?;
                    let mut de = serde_json::Deserializer::from_str(&content);
                    let pkg = AdlPackage::deserialize(&mut de)
                        .map_err(|e| anyhow!("{:?}: {}", p_path, e.to_string()))?;
                    wrk1.r#use.push(Payload1::new(p.clone(), pkg));
                }
                // println!("wrk {:?}", wrk1);
                return Ok((porw.1, wrk1));
            }
        }
    }
    Err(anyhow!("No workspace found"))
}

// const ADL_PKG_FILES: &[(&str, PkgDef)] = &[
//     ("adl.pkg.json", PkgDef::Pkg),
//     ("adl.work.json", PkgDef::Work),
// ];

#[derive(Debug, Copy, Clone, PartialEq)]
enum PkgDef {
    Pkg,
    Work,
}

fn collect_work_and_pkg(
    opts: &super::GenOpts,
) -> Result<Vec<(PkgDef, PathBuf, &str)>, anyhow::Error> {
    let mut res = vec![];
    let current_dir = env::current_dir()?;
    let current_dir = current_dir.join(&opts.dir);
    let mut current_dir = current_dir.canonicalize()?;

    loop {
        {
            let f = (&opts.workspace_filename, PkgDef::Work);
            let file_path = current_dir.join(f.0);
            if file_path.exists() {
                res.push((f.1, current_dir.clone(), f.0.as_str()));
            }
        }
        {
            let f = (&opts.package_filenames, PkgDef::Pkg);
            let file_path = current_dir.join(f.0);
            if file_path.exists() {
                res.push((f.1, current_dir.clone(), f.0.as_str()));
            }
        }
        if !current_dir.pop() {
            break;
        }
    }
    Ok(res)
}
