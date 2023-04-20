use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs};

use anyhow::anyhow;

use serde::Deserialize;

use crate::adlgen::adlc::packaging::{
    AdlPackage, AdlWorkspace0, AdlWorkspace1, Payload1
};
use crate::processing::loader::loader_from_workspace;

use super::tsgen;

pub(crate) fn workspace(opts: &super::GenOpts) -> Result<(), anyhow::Error> {
    let pkg_defs = collect_work_and_pkg(opts)?;
    let wrk1 = collection_to_workspace(pkg_defs)?;
    // println!("{:?}", &wrk1);
    for pkg in &wrk1.1.r#use {
        if let Some(opts) = &pkg.p_ref.ts_opts {
            let loader = loader_from_workspace(wrk1.0.clone(), wrk1.1.clone());
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
        match  rt {
            crate::adlgen::adlc::packaging::RuntimeOpts::TsRuntime(rt_opts) => {
                tsgen::write_runtime(rt_opts)?
            },
        }
    }
    Ok(())
}

fn tuple_str_to_map(arg: &[(&str, &str)]) -> HashMap<String, String> {
    arg.iter()
        .map(|a| (String::from(a.0), String::from(a.1)))
        .collect()
}

// fn embedded_payload() -> Option<Payload1> {
//     Some(Payload1 {
//         p_ref: AdlPackageRef {
//             path: "".to_string(),
//             ts_opts: Some(TypescriptGenOptions {
//                 npm_pkg_name: "@adl-lang/sys".to_string(),
//                 npm_version: TypescriptGenOptions::def_npm_version(),
//                 extra_dependencies: tuple_str_to_map(&[("base64-js", "^1.5.1")]),
//                 extra_dev_dependencies: tuple_str_to_map(&[
//                     ("tsconfig", "workspace:*"),
//                     ("typescript", "^4.9.3"),
//                 ]),
//                 outputs: TypescriptGenOptions::def_outputs(),
//                 runtime_opts: TypescriptGenOptions::def_runtime_opts(),
//                 generate_transitive: false,
//                 include_resolver: false,
//                 ts_style: TypescriptGenOptions::def_ts_style(),
//                 modules: TypescriptGenOptions::def_modules(),
//                 capitalize_branch_names_in_types:
//                     TypescriptGenOptions::def_capitalize_branch_names_in_types(),
//                 capitalize_type_names: TypescriptGenOptions::def_capitalize_type_names(
//                 ),
//                 annotate: TypescriptGenOptions::def_annotate(),
//             }),
//         },
//         pkg: AdlPackage {
//             path: "github.com/adl-lang/adl/adl/stdlib/sys".to_string(),
//             global_alias: Some("sys".to_string()),
//             adlc: "0.0.0".to_string(),
//             requires: vec![],
//             excludes: vec![],
//             replaces: vec![],
//             retracts: vec![],
//         },
//     })
// }

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
                    embedded_sys_loader: wrk0.embedded_sys_loader,
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
