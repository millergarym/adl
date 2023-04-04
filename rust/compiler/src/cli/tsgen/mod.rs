use super::TsOpts;

extern crate regex;

use regex::bytes::Regex;

use std::collections::HashMap;
use std::ops::Deref;
use std::path::PathBuf;

use anyhow::anyhow;

use genco::fmt::{self, Indentation};
use genco::prelude::*;

use crate::adlgen::sys::adlast2::{self as adlast};
use crate::adlgen::sys::adlast2::{Module, Module1, TypeExpr, TypeRef};
use crate::processing::loader::loader_from_search_paths;
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

pub fn tsgen(opts: &TsOpts) -> anyhow::Result<()> {
    let loader = loader_from_search_paths(&opts.search.path);
    let mut resolver = Resolver::new(loader);
    for m in &opts.modules {
        let r = resolver.add_module(m);
        match r {
            Ok(()) => (),
            Err(e) => return Err(anyhow!("Failed to load module {}: {:?}", m, e)),
        }
    }

    let manifest = opts.output.manifest.clone();
    // let manifest = opts.output.manifest.as_ref().map(|m| {
    //     if m.extension() == None {
    //         let mut m0 = m.clone();
    //         m0.set_extension("json");
    //         m0
    //     } else {
    //         m.clone()
    //     }
    // });

    let mut writer = TreeWriter::new(opts.output.outdir.clone(), manifest)?;

    if !opts.include_runtime {
        if opts.runtime_dir == None || opts.ts_style == None {
            return Err(anyhow!("Invalid flags; --runtime-dir and --ts-style only valid if --include-runtime is set"));
        }
    }

    let modules: Vec<&Module1> = resolver
        .get_module_names()
        .into_iter()
        .map(|mn| resolver.get_module(&mn).unwrap())
        .collect();

    for m in modules {
        let path = path_from_module_name(opts, m.name.to_owned());
        let code = gen_ts_module(m, &resolver, opts)?;
        writer.write(path.as_path(), code)?;
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

    if opts.include_runtime {
        gen_runtime(opts, &mut writer)?
    }

    Ok(())
}

fn gen_ts_module(
    m: &Module<TypeExpr<TypeRef>>,
    resolver: &Resolver,
    opts: &TsOpts,
) -> anyhow::Result<String> {
    // TODO sys.annotations::SerializedName needs to be embedded
    let tokens = &mut js::Tokens::new();
    let mut mgen = generate::TsGenVisitor {
        module: m,
        resolver: resolver,
        adlr: js::import(
            utils::rel_import(&m.name, &"runtime.adl".to_string()),
            "ADL",
        )
        .into_wildcard(),
        map: &mut HashMap::new(),
        opts,
    };
    mgen.gen_module(tokens, m)?;
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

fn path_from_module_name(_opts: &TsOpts, mname: adlast::ModuleName) -> PathBuf {
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

fn gen_runtime(opts: &TsOpts, writer: &mut TreeWriter) -> anyhow::Result<()> {
    let re = Regex::new(r"\$TSEXT").unwrap();
    let re2 = Regex::new(r"\$TSB64IMPORT").unwrap();
    for rt in RUNTIME.iter() {
        let mut file_path = PathBuf::new();
        if let Some(d) = &opts.runtime_dir {
            file_path.push(d);
        } else {
            file_path.push("runtime");
        };
        file_path.push(rt.0);
        let dir_path = file_path.parent().unwrap();
        std::fs::create_dir_all(dir_path)?;

        log::info!("writing {}", file_path.display());
        eprintln!("writing runtime file '{}'", file_path.display());

        let tss = if let Some(tss) = opts.ts_style {
            tss
        } else {
            super::TsStyle::Tsc
        };
        match tss {
            super::TsStyle::Tsc => {
                let content = re.replace_all(rt.1, "".as_bytes());
                let content = re2.replace(&content, TSC_B64);
                let x = content.deref();
                let y = String::from_utf8(x.to_vec())?;
                writer.write(file_path.as_path(), y)?;
                // std::fs::write(file_path, content)
                //     .map_err(|e| anyhow!("error writing runtime file {}", e.to_string()))?;
            }
            super::TsStyle::Deno => {
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
