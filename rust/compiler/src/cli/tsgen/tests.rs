use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use serde::Deserialize;

use crate::adlgen_dev::testing_table::TestFilesMetaData;
use crate::cli::{AdlSearchOpts, OutputOpts};

use super::*;

#[test]
fn generate_ts_from_test_files() {
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
                let mut outdir = PathBuf::from("build/dev_adl");
                outdir.push(t.module_root.clone());
                let mut manifest = PathBuf::from("build/dev_adl");
                manifest.push(t.module_root.clone());
                manifest.push("manifest");

                let mut search_path = vec![];
                search_path.push(PathBuf::from("../../adl/stdlib"));
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
                let opts = TsOpts {
                    search: AdlSearchOpts { path: search_path },
                    output: OutputOpts {
                        outdir,
                        manifest: Some(manifest),
                    },
                    include_runtime: true,
                    runtime_dir: Some("runtime".to_string()),
                    ts_style: None,
                    modules: t.modules.clone(),
                    capitalize_branch_names_in_types: true,
                };
                if !t.fail && !t.skip {
                    let mut adlc_cmd = String::new();
                    adlc_cmd.push_str("adlc typescript");
                    opts.search.path.iter().for_each(|p| {
                        adlc_cmd.push_str(" --searchdir=");
                        adlc_cmd.push_str(p.to_str().unwrap());
                    });
                    adlc_cmd.push_str(" --outputdir=build/adlc_out/");
                    adlc_cmd.push_str(t.module_root.clone().as_str());
                    // adlc_cmd.push_str(opts.output.outdir.to_str().unwrap());
                    adlc_cmd.push_str(" --generate-transitive");
                    adlc_cmd.push_str(" --include-rt");
                    adlc_cmd.push_str(" --include-resolver");
                    adlc_cmd.push_str(" --runtime-dir=runtime");

                    adlc_cmd.push_str(" --manifest=");
                    adlc_cmd.push_str("build/adlc_out/");
                    adlc_cmd.push_str(t.module_root.clone().as_str());
                    adlc_cmd.push_str("/manifest");

                    opts.modules.iter().for_each(|m| {
                        adlc_cmd.push_str(" ");
                        adlc_cmd.push_str("../../adl/tests/");
                        adlc_cmd.push_str(t.module_root.clone().as_str());
                        adlc_cmd.push_str("/");
                        adlc_cmd.push_str(m);
                        adlc_cmd.push_str(".adl");
                    });
                    adlc_cmds.push(adlc_cmd);
                    // println!("{}", adlc_cmd);
                }

                // TODO consider failed.
                // t.fail
                match tsgen(&opts) {
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
                            assert!(false, "the above test was expected to fail, but passed")
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
                            assert!(false, "Error : '{:?}'\n{}", t, e.to_string());
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
