use std::io::BufReader;
use std::path::PathBuf;
use std::{collections::BTreeSet, fs::File};

use serde::Deserialize;

use crate::{
    adlgen::adlc::{
        testing_table::TestFilesMetaData,
        workspace::{DirectoryRef, GenOutput, ModuleSrc, ReferenceableScopeOption, TsGenRuntime},
    },
    cli::formatter,
    processing::loader::loader_from_search_paths,
};

use super::*;

#[test]
fn generate_ts_from_test_files() {
    let _ = env_logger::builder()
        .is_test(true)
        .format(formatter)
        .filter_level(log::LevelFilter::Info)
        .try_init();

    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("../../adl/tests/testing_table.json");

    let file = File::open(d).expect(&format!(
        "Failed to read file: {}",
        "../../adl/tests/testing_table.json"
    ));
    let reader = BufReader::new(file);

    let mut de = serde_json::Deserializer::from_reader(reader);
    let mut adlc_cmds = vec![];
    match TestFilesMetaData::deserialize(&mut de) {
        Ok(tests) => {
            for t in &tests {
                if t.skip {
                    println!("Skipping {} {} - ts gen output;", &t.module_root, &t.title,);
                    continue;
                }
                let outdir = match &t.output_dir {
                    Some(dir) => dir.clone(),
                    None => {
                        let mut outdir = String::from("build/dev_adl/");
                        outdir.push_str(t.module_root.as_str());
                        outdir.clone()
                    }
                };

                let manifest = match &t.output_dir {
                    Some(dir) => {
                        let mut manifest = String::from(dir);
                        manifest.push_str("manifest");
                        manifest
                    }
                    None => {
                        let mut manifest = String::from("build/dev_adl/");
                        manifest.push_str(t.module_root.as_str());
                        manifest.push_str("/manifest");
                        manifest
                    }
                };

                let mut search_path = vec![];
                search_path.push(PathBuf::from("../../adl/stdlib"));
                search_path.push(PathBuf::from("../../adl/tools"));
                {
                    let mut sp = PathBuf::from("../../adl/tests/");
                    sp.push(t.module_root.clone());
                    search_path.push(sp);
                }
                {
                    t.lib_paths.iter().for_each(|lib| {
                        let mut sp = PathBuf::from("../../adl/tests/");
                        sp.push(lib);
                        search_path.push(sp);
                    })
                }
                let modules = t.modules.clone();
                let ts_opts = TypescriptGenOptions {
                    npm_pkg_name: "testing".to_string(),
                    npm_version: "0.0.0".to_string(),
                    scripts: TypescriptGenOptions::def_scripts(),
                    tsconfig: TypescriptGenOptions::def_tsconfig(),
                    extra_dependencies: TypescriptGenOptions::def_extra_dependencies(),
                    extra_dev_dependencies: TypescriptGenOptions::def_extra_dev_dependencies(),
                    outputs: Some(crate::adlgen::adlc::workspace::OutputOpts::Gen(GenOutput {
                        referenceable: ReferenceableScopeOption::Local,
                        output_dir: outdir.clone(),
                        manifest: Some(manifest),
                    })),
                    runtime_opts: TsRuntimeOpt::Generate(TsGenRuntime {}),
                    generate_transitive: true,
                    include_resolver: true,
                    ts_style: crate::adlgen::adlc::workspace::TsStyle::Tsc,
                    modules: ModuleSrc::Modules(t.modules.clone()),
                    capitalize_branch_names_in_types: true,
                    capitalize_type_names: true,
                };

                let mut adlc_cmd = String::new();
                adlc_cmd.push_str("adlc typescript");
                search_path.iter().for_each(|p| {
                    adlc_cmd.push_str(" --searchdir=");
                    adlc_cmd.push_str(p.to_str().unwrap());
                });
                adlc_cmd.push_str(" --outputdir=");
                // if let Some(dir) = &t.output_dir {
                //     adlc_cmd.push_str(dir.as_str());
                // } else {
                adlc_cmd.push_str("build/adlc_out/");
                adlc_cmd.push_str(t.module_root.clone().as_str());
                // }
                // adlc_cmd.push_str(opts.output.outdir.to_str().unwrap());
                adlc_cmd.push_str(" --generate-transitive");
                adlc_cmd.push_str(" --include-rt");
                adlc_cmd.push_str(" --include-resolver");
                adlc_cmd.push_str(" --runtime-dir=runtime");

                adlc_cmd.push_str(" --manifest=");
                // if let Some(dir) = &t.output_dir {
                //     adlc_cmd.push_str(dir.as_str());
                //     adlc_cmd.push_str("/manifest");
                // } else {
                adlc_cmd.push_str("build/adlc_out/");
                adlc_cmd.push_str(t.module_root.clone().as_str());
                adlc_cmd.push_str("/manifest");
                // }

                modules.iter().for_each(|m| {
                    adlc_cmd.push_str(" ");
                    adlc_cmd.push_str("../../adl/tests/");
                    adlc_cmd.push_str(t.module_root.clone().as_str());
                    adlc_cmd.push_str("/");
                    adlc_cmd.push_str(m.replace(".", "/").as_str());
                    adlc_cmd.push_str(".adl");
                });
                if !t.fail && !t.skip {
                    adlc_cmds.push(adlc_cmd.clone());
                    // println!("{}", adlc_cmd);
                }

                // TODO consider failed.
                // t.fail
                // let dep_adl_bundles = vec![];
                let bundle = t
                    .bundle_file.clone()
                    .map(|p| {
                        let mut unused = BTreeSet::new();

                        let mut porw_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                        porw_path.push("../../adl/tests/");

                        porw_path.push(t.module_root.clone());
                        porw_path.push(p.clone());
                        let content = fs::read_to_string(&porw_path)
                            .map_err(|e| anyhow!("{:?}: {}", porw_path, e.to_string()))?;
                        let de = &mut serde_json::Deserializer::from_str(&content);

                        let b0: AdlBundle = serde_ignored::deserialize(de, |path| {
                            unused.insert(path.to_string());
                        })
                        .map_err(|e| anyhow!("{:?}: {}", p.clone(), e.to_string()))?;

                        if unused.len() != 0 {
                            return Err(anyhow!("unknown fields `{:?}` {:?}", unused, p));
                        }
                        Ok(b0)
                    })
                    .map_or(Ok(None), |v| v.map(Some));
                let bundle1 = match bundle {
                    Ok(b) => b,
                    Err(e) => {
                        assert!(false, "error parsing bundle {}", e);
                        return ();
                    }
                };
                match tsgen(
                    false,
                    false,
                    loader_from_search_paths(&search_path),
                    bundle1,
                    &ts_opts,
                    None,
                    AdlBundleRefType::Dir(".".to_string()),
                    // dep_adl_bundles,
                ) {
                    Ok(_) => {
                        println!(
                            "{} {} - ts gen output;  {}",
                            &t.module_root,
                            &t.title,
                            &t.description.join("\n    "),
                        );
                        for m in &t.modules {
                            println!("  build/dev_adl/{}/{}.ts", &t.module_root, m)
                        }
                        if t.fail {
                            assert!(false, "the above test was expected to fail, but passed.\nadlc command would be:\n\t{}\n", adlc_cmd.clone())
                        }
                    }
                    Err(e) => {
                        if t.fail {
                            println!(
                                "{} {} - failed as expected for src;  {}",
                                &t.module_root,
                                &t.title,
                                &t.description.join("\n\t"),
                            );
                            for m in &t.modules {
                                println!("  ../../adl/tests/{}/{}.adl", &t.module_root, m)
                            }
                            println!("  error '{}'", e.to_string());
                        } else {
                            println!(
                                "{} {} {} - Error '{}'",
                                &t.module_root,
                                &t.title,
                                &t.description.join("\n\t"),
                                e.to_string()
                            );
                            for m in &t.modules {
                                println!("  ../../adl/tests/{}/{}.adl", &t.module_root, m)
                            }
                            assert!(
                                false,
                                "Error : '{:?}'\n{}\nadlc command would be:\n\t{}\n",
                                t,
                                e.to_string(),
                                adlc_cmds[adlc_cmds.len() - 1]
                            );
                        }
                    }
                };
            }
        }
        Err(err) => assert!(false, "error deserializing testing_table {}", err),
    }
    adlc_cmds.iter().for_each(|cmd| {
        println!("{}", cmd);
    })

    // // Read the JSON contents of the file as an instance of `User`.
    // let u: Result<TestFilesMetaData, _> = serde_json::from_reader(reader);
    // match u {
    //     Ok(tests) => { dbg!(tests); },
    //     Err(err) => assert!(false, "error deserializing testing_table {}", err),
    // }
}
