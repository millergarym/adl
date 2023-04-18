extern crate regex;

use regex::bytes::Regex;

use std::collections::HashMap;
use std::fs;
use std::ops::Deref;
use std::path::PathBuf;

use anyhow::anyhow;

use genco::fmt::{self, Indentation};
use genco::prelude::*;

use crate::adlgen::adlc::packaging::{TsGenRuntime, TsRuntimeOpt, TsStyle, TypescriptGenOptions};
use crate::adlgen::sys::adlast2::{self as adlast};
use crate::adlgen::sys::adlast2::{Module, Module1, TypeExpr, TypeRef};
use crate::processing::loader::{loader_from_search_paths, AdlLoader};
use crate::processing::resolver::Resolver;
use crate::processing::writer::TreeWriter;

mod astgen;
mod defaultval;
mod generate;
#[cfg(test)]
mod tests;
mod utils;

const RUNTIME_JSON: &[u8] = include_bytes!("runtime/json.ts");
const RUNTIME_ADL: &[u8] = include_bytes!("runtime/adl.ts");
const RUNTIME_UTILS: &[u8] = include_bytes!("runtime/utils.ts");
const RUNTIME_DYNAMIC: &[u8] = include_bytes!("runtime/dynamic.ts");
const RUNTIME_SYS_DYNAMIC: &[u8] = include_bytes!("runtime/sys/dynamic.ts");
const RUNTIME_SYS_ADLAST: &[u8] = include_bytes!("runtime/sys/adlast.ts");
const RUNTIME_SYS_TYPES: &[u8] = include_bytes!("runtime/sys/types.ts");

const RUNTIME: [&'static (&str, &[u8]); 7] = [
    &("json.ts", RUNTIME_JSON),
    &("adl.ts", RUNTIME_ADL),
    &("utils.ts", RUNTIME_UTILS),
    &("dynamic.ts", RUNTIME_DYNAMIC),
    &("sys/dynamic.ts", RUNTIME_SYS_DYNAMIC),
    &("sys/adlast.ts", RUNTIME_SYS_ADLAST),
    &("sys/types.ts", RUNTIME_SYS_TYPES),
];

const TSC_B64: &[u8] =
    b"import {fromByteArray as b64Encode, toByteArray as b64Decode} from 'base64-js'";
const DENO_B64: &[u8] = b"import {encode as b64Encode, decode as b64Decode} from 'https://deno.land/std@0.97.0/encoding/base64.ts'";

fn get_modules(
    opts: &TypescriptGenOptions,
    pkg_root: Option<PathBuf>,
) -> Result<Vec<String>, anyhow::Error> {
    match &opts.modules {
        crate::adlgen::adlc::packaging::ModuleSrc::All => {
            let pkg_root = if let Some(pkg_root) = pkg_root {
                pkg_root
            } else {
                return Err(anyhow!("pkg_root needed when module src all specified"));
            };
            // let pkg_root = wrk1.0.join(pkg.0 .0.path.clone()).canonicalize()?;
            // let pkg_root = wrk1_path.join(pkg_path).canonicalize()?;
            if let Some(pkg_root_str) = pkg_root.as_os_str().to_str() {
                Ok(walk_and_collect_adl_modules(pkg_root_str, &pkg_root))
            } else {
                return Err(anyhow!("Could get str from pkg_root"));
            }
        }
        crate::adlgen::adlc::packaging::ModuleSrc::Modules(ms) => Ok(ms.clone()),
    }
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

pub fn tsgen(
    loader: Box<dyn AdlLoader>,
    opts: &TypescriptGenOptions,
    pkg_root: Option<PathBuf>,
) -> anyhow::Result<()> {
    let (manifest, outputdir) = match &opts.outputs {
        crate::adlgen::adlc::packaging::OutputOpts::Ref(_) => return Ok(()),
        crate::adlgen::adlc::packaging::OutputOpts::Gen(gen) => (
            gen.manifest.as_ref().map(|m| PathBuf::from(m)),
            PathBuf::from(gen.output_dir.clone()),
        ),
    };

    let mut resolver = Resolver::new(loader);
    let module_names = get_modules(opts, pkg_root)?;
    for m in &module_names {
        let r = resolver.add_module(m);
        match r {
            Ok(()) => (),
            Err(e) => return Err(anyhow!("Failed to load module {}: {:?}", m, e)),
        }
    }

    let mut writer = TreeWriter::new(outputdir, manifest)?;

    let modules: Vec<&Module1> = resolver
        .get_module_names()
        .into_iter()
        .map(|mn| resolver.get_module(&mn).unwrap())
        .collect();

    for m in modules {
        if opts.generate_transitive || module_names.contains(&m.name) {
            let path = path_from_module_name(opts, m.name.to_owned());
            let code = gen_ts_module(m, &resolver, opts)?;
            writer.write(path.as_path(), code)?;
        }
    }

    {
        let tokens = &mut js::Tokens::new();
        let modules = resolver.get_module_names();
        gen_resolver(tokens, modules)?;
        let config = js::Config::default();
        // let config = js::Config{
        //     ..Default::default()
        // };
        let mut w = fmt::IoWriter::new(Vec::<u8>::new());
        // let mut w = fmt::IoWriter::new(stdout.lock());
        let fmt = fmt::Config::from_lang::<JavaScript>();
        let fmt = fmt::Config::with_indentation(fmt, Indentation::Space(2));
        tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
        let vector = w.into_inner();
        let code = std::str::from_utf8(&vector)?;
        let path = path_from_module_name(opts, "resolver".to_string());
        writer.write(path.as_path(), code.to_string())?;
    }

    if let TsRuntimeOpt::Generate(rt_gen_opts) = &opts.runtime_opts {
        gen_runtime(rt_gen_opts, &opts.ts_style, &mut writer)?
    }

    Ok(())
}

fn gen_ts_module(
    m: &Module1,
    resolver: &Resolver,
    opts: &TypescriptGenOptions,
) -> anyhow::Result<String> {
    // TODO sys.annotations::SerializedName needs to be embedded
    let tokens = &mut js::Tokens::new();
    // match opts.runtime_opts {
    //     TsRuntimeOpt::PackageRef(pkg) => todo!(),
    //     TsRuntimeOpt::Generate(_) => todo!(),
    // }
    let adlr = match &opts.runtime_opts {
        TsRuntimeOpt::PackageRef(pkg) => {
            // sdf
            js::import(pkg.clone() + "/adl", "ADL").into_wildcard()
        }
        TsRuntimeOpt::Generate(gen) => {
            // TODO modify the import path with opts.runtime_dir
            js::import(
                utils::rel_import(&m.name, &"runtime.adl".to_string()),
                "ADL",
            )
            .into_wildcard()
        }
    };
    let mut mgen = generate::TsGenVisitor {
        module: m,
        npm_pkg: &None,
        resolver: resolver,
        adlr,
        map: &mut HashMap::new(),
        opts,
    };
    mgen.gen_module(tokens)?;
    // let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(Vec::<u8>::new());
    // let mut w = fmt::IoWriter::new(stdout.lock());
    let fmt = fmt::Config::from_lang::<JavaScript>();
    let fmt = fmt::Config::with_indentation(fmt, Indentation::Space(2));

    let config = js::Config::default();
    // let config = js::Config{
    //     ..Default::default()
    // };
    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
    let vector = w.into_inner();
    let code = std::str::from_utf8(&vector)?;
    // let code = tokens.to_file_string()?;
    // tokens.format_file(out, config);
    Ok(code.to_string())
}

fn path_from_module_name(_opts: &TypescriptGenOptions, mname: adlast::ModuleName) -> PathBuf {
    let mut path = PathBuf::new();
    // path.push(opts.module.clone());
    for el in mname.split(".") {
        path.push(el);
    }
    path.set_extension("ts");
    return path;
}

fn gen_resolver(t: &mut Tokens<JavaScript>, mut modules: Vec<String>) -> anyhow::Result<()> {
    let mut m_imports = vec![];
    for m in &modules {
        m_imports.push(
            js::import(format!("./{}", m.replace(".", "/")), "_AST_MAP")
                .with_alias(m.replace(".", "_")),
        );
    }
    let adlr1 = js::import("./runtime/adl", "declResolver");
    let adlr2 = js::import("./runtime/adl", "ScopedDecl");
    let gened = "/* @generated from adl */";
    modules.sort();
    quote_in! { *t =>
    $gened
    $(register (adlr2))
    $(register (adlr1))
    $(for m in m_imports => $(register (m)))


    export const ADL: { [key: string]: ScopedDecl } = {
      $(for m in &modules => ...$(m.replace(".", "_")),$['\r'])
    };

    export const RESOLVER = declResolver(ADL);
    }

    Ok(())
}

fn gen_runtime(
    rt_gen_opts: &TsGenRuntime,
    ts_style: &TsStyle,
    writer: &mut TreeWriter,
) -> anyhow::Result<()> {
    let re = Regex::new(r"\$TSEXT").unwrap();
    let re2 = Regex::new(r"\$TSB64IMPORT").unwrap();
    for rt in RUNTIME.iter() {
        let mut file_path = PathBuf::new();
        file_path.push(&rt_gen_opts.runtime_dir);
        file_path.push(rt.0);
        let dir_path = file_path.parent().unwrap();
        std::fs::create_dir_all(dir_path)?;

        log::info!("writing {}", file_path.display());

        match ts_style {
            TsStyle::Tsc => {
                let content = re.replace_all(rt.1, "".as_bytes());
                let content = re2.replace(&content, TSC_B64);
                let x = content.deref();
                let y = String::from_utf8(x.to_vec())?;
                writer.write(file_path.as_path(), y)?;
                // std::fs::write(file_path, content)
                //     .map_err(|e| anyhow!("error writing runtime file {}", e.to_string()))?;
            }
            TsStyle::Deno => {
                let content = re.replace_all(rt.1, ".ts".as_bytes());
                let content = re2.replace(&content, DENO_B64);
                let x = content.deref();
                let y = String::from_utf8(x.to_vec())?;
                writer.write(file_path.as_path(), y)?;
            }
        }
    }
    Ok(())
}
