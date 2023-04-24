use regex;

use regex::bytes::Regex;

use std::collections::HashMap;
use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use anyhow::anyhow;

use genco::fmt::{self, Indentation};
use genco::prelude::*;

use crate::adlgen::adlc::packaging::{
    AdlWorkspace, NpmPackage, NpmPackageRef, Payload1, TsGenRuntime, TsRuntimeOpt, TsStyle,
    TsWriteRuntime, TypescriptGenOptions,
};
use crate::adlgen::sys::adlast2::Module1;
use crate::adlgen::sys::adlast2::{self as adlast};
use crate::cli::tsgen::utils::{get_npm_pkg, npm_pkg_import};
use crate::processing::loader::AdlLoader;
use crate::processing::resolver::Resolver;
use crate::processing::writer::TreeWriter;

mod astgen;
mod defaultval;
mod generate;
#[cfg(test)]
mod tests;
mod utils;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "src/cli/tsgen/ts-runtime/"]
struct Asset;

// const RUNTIME_PACKAGE: &[u8] = include_bytes!("runtime/package.json");
// const RUNTIME_JSON: &[u8] = include_bytes!("runtime/json.ts");
// const RUNTIME_ADL: &[u8] = include_bytes!("runtime/adl.ts");
// const RUNTIME_UTILS: &[u8] = include_bytes!("runtime/utils.ts");
// const RUNTIME_DYNAMIC: &[u8] = include_bytes!("runtime/dynamic.ts");
// const RUNTIME_SYS_DYNAMIC: &[u8] = include_bytes!("runtime/sys/dynamic.ts");
// const RUNTIME_SYS_ADLAST: &[u8] = include_bytes!("runtime/sys/adlast.ts");
// const RUNTIME_SYS_TYPES: &[u8] = include_bytes!("runtime/sys/types.ts");

// const RUNTIME: [&'static (&str, &[u8]); 8] = [
//     &("package.json", RUNTIME_PACKAGE),
//     &("json.ts", RUNTIME_JSON),
//     &("adl.ts", RUNTIME_ADL),
//     &("utils.ts", RUNTIME_UTILS),
//     &("dynamic.ts", RUNTIME_DYNAMIC),
//     &("sys/dynamic.ts", RUNTIME_SYS_DYNAMIC),
//     &("sys/adlast.ts", RUNTIME_SYS_ADLAST),
//     &("sys/types.ts", RUNTIME_SYS_TYPES),
// ];

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
    if opts.outputs == None {
        // not gen for this pkg
        return Ok(());
    }
    let outputs = opts.outputs.as_ref().unwrap();
    let (manifest, outputdir, strip_first) = match outputs {
        crate::adlgen::adlc::packaging::OutputOpts::Gen(gen) => (
            gen.manifest.as_ref().map(|m| PathBuf::from(m)),
            PathBuf::from(gen.output_dir.clone()),
            gen.strip_first,
        ),
    };

    let mut resolver = Resolver::new(loader);
    let module_names = get_modules(opts, pkg_root)?;
    println!("module_names:{:?}", module_names);
    for m in &module_names {
        let r = resolver.add_module(m);
        match r {
            Ok(()) => (),
            Err(e) => return Err(anyhow!("Failed to load module {}: {:?}", m, e)),
        }
    }

    let mut writer = TreeWriter::new(outputdir.clone(), manifest)?;

    println!("----");
    let parent = outputdir.file_name().unwrap().to_str().unwrap().to_string();
    println!("!!!'{}'", parent);

    let modules: Vec<&Module1> = resolver
        .get_module_names()
        .into_iter()
        .map(|mn| resolver.get_module(&mn).unwrap())
        .collect();

    for m in &modules {
        if opts.generate_transitive || module_names.contains(&m.name) {
            let path = path_from_module_name(strip_first, m.name.to_owned());
            println!("~~~{} - {:?}", m.name.to_owned(), path);
            let code = gen_ts_module(m, &resolver, opts)?;
            writer.write(path.as_path(), code)?;
        }
    }

    {
        let tokens = &mut js::Tokens::new();
        // let modules = resolver.get_module_names();
        if opts.include_resolver {
            gen_resolver(
                tokens,
                opts.npm_pkg_name.clone(),
                opts.generate_transitive,
                &opts.runtime_opts,
                &resolver,
                &modules,
            )?;
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
            let path = path_from_module_name(false, "resolver".to_string());
            writer.write(path.as_path(), code.to_string())?;
        }
    }

    if let TsRuntimeOpt::Generate(_) = &opts.runtime_opts {
        gen_runtime(strip_first, &opts.ts_style, &mut writer)?
    }

    Ok(())
}

pub fn gen_npm_package(pkg_path: String, wrk1: &AdlWorkspace<Payload1>) -> anyhow::Result<()> {
    let payload = wrk1
        .r#use
        .iter()
        .find(|w| w.p_ref.path == pkg_path)
        .unwrap();
    let opts = payload.p_ref.ts_opts.as_ref().unwrap();

    if opts.outputs == None {
        // not gen for this pkg
        return Ok(());
    }
    let outputs = opts.outputs.as_ref().unwrap();
    let outputdir = match outputs {
        crate::adlgen::adlc::packaging::OutputOpts::Gen(gen) => {
            PathBuf::from(gen.output_dir.clone())
        }
    };
    let mut writer = TreeWriter::new(outputdir.clone(), None)?;

    let mut npm_package = NpmPackage::new(opts.npm_pkg_name.clone(), opts.npm_version.clone());
    match &opts.runtime_opts {
        TsRuntimeOpt::WorkspaceRef(rt) => {
            npm_package
                .dependencies
                .insert(rt.clone(), "workspace:*".to_string());
        }
        TsRuntimeOpt::PackageRef(rt) => {
            npm_package
                .dependencies
                .insert(rt.name.clone(), rt.version.clone());
        }
        TsRuntimeOpt::Generate(_) => {}
    };

    for d in &opts.extra_dependencies {
        npm_package.dependencies.insert(d.0.clone(), d.1.clone());
    }
    for d in &opts.extra_dev_dependencies {
        npm_package
            .dev_dependencies
            .insert(d.0.clone(), d.1.clone());
    }

    if !opts.generate_transitive {
        for r in payload.pkg.requires.iter() {
            match &r.r#ref {
                crate::adlgen::adlc::packaging::PkgRef::Path(p0) => {
                    match wrk1.r#use.iter().find(|p| p.pkg.path == *p0) {
                        Some(p1) => match &p1.p_ref.ts_opts {
                            Some(ts_opts) => {
                                npm_package.dependencies.insert(
                                    ts_opts.npm_pkg_name.clone(),
                                    "workspace:*".to_string(),
                                );
                            }
                            None => {
                                return Err(anyhow!(
                                    "pkg_ref::path - no ts_opts in workspace file for package '{}'",
                                    p1.p_ref.path
                                ))
                            }
                        },
                        None => return Err(anyhow!("no package is workspace with path '{}'", p0)),
                    }
                }
                crate::adlgen::adlc::packaging::PkgRef::Alias(a) => {
                    match wrk1
                        .r#use
                        .iter()
                        .find(|p| p.pkg.global_alias == Some(a.to_string()))
                    {
                        Some(p1) => match &p1.p_ref.ts_opts {
                            Some(ts_opts) => {
                                npm_package.dependencies.insert(
                                    ts_opts.npm_pkg_name.clone(),
                                    "workspace:*".to_string(),
                                );
                            }
                            None => {
                                return Err(anyhow!(
                                "pkg_ref::alias - no ts_opts in workspace file for package '{}'",
                                p1.p_ref.path
                            ))
                            }
                        },
                        None => {
                            if *a == "sys".to_string() {
                                npm_package
                                    .dependencies
                                    .insert("@adl-lang/sys".to_string(), "1.0.0".to_string());
                            } else {
                                return Err(anyhow!("no package is workspace with alias '{}'", a));
                            }
                        }
                    }
                }
            };
        }
    }
    let content = serde_json::to_string_pretty(&npm_package)?;
    writer.write(Path::new("package.json"), content)?;
    eprintln!("generated {:?}", outputdir.clone().join("package.json"));

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
        TsRuntimeOpt::WorkspaceRef(pkg) => js::import(pkg.clone() + "/adl", "ADL").into_wildcard(),
        TsRuntimeOpt::PackageRef(pkg) => {
            js::import(pkg.name.clone() + "/adl", "ADL").into_wildcard()
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

fn path_from_module_name(strip_first: bool, mname: adlast::ModuleName) -> PathBuf {
    let mut path = PathBuf::new();
    for (i, el) in mname.split(".").enumerate() {
        if i == 0 && strip_first {
            continue;
        }
        path.push(el);
    }
    path.set_extension("ts");
    return path;
}

fn gen_resolver(
    t: &mut Tokens<JavaScript>,
    npm_pkg: String,
    generate_transitive: bool,
    runtime_opts: &TsRuntimeOpt,
    resolver: &Resolver,
    modules: &Vec<&Module1>,
) -> anyhow::Result<()> {
    // TODO remote or local imports
    let mut m_imports = vec![];

    for m in modules {
        let npm_pkg2 = if let Some(m2) = resolver.get_module(&m.name) {
            get_npm_pkg(&m2)
        } else {
            None
        };

        let path = if !generate_transitive && npm_pkg2 != None {
            let npm_pkg2 = npm_pkg2.unwrap();
            if npm_pkg2 != npm_pkg {
                npm_pkg_import(npm_pkg2, m.name.clone())
            } else {
                format!("./{}", m.name.replace(".", "/"))
            }
        } else {
            format!("./{}", m.name.replace(".", "/"))
        };

        m_imports.push(js::import(path, "_AST_MAP").with_alias(m.name.replace(".", "_")));
    }

    let (adlr1, adlr2) = match runtime_opts {
        TsRuntimeOpt::WorkspaceRef(pref) => (
            js::import(format!("{}/adl", pref.as_str()), "declResolver"),
            js::import(format!("{}/adl", pref.as_str()), "ScopedDecl"),
        ),
        TsRuntimeOpt::PackageRef(pref) => (
            js::import(format!("{}/adl", pref.name.as_str()), "declResolver"),
            js::import(format!("{}/adl", pref.name.as_str()), "ScopedDecl"),
        ),
        TsRuntimeOpt::Generate(gen) => {
            // js::import(format!("{}/adl", gen.runtime_dir), "declResolver"),
            // js::import(format!("{}/adl", gen.runtime_dir), "ScopedDecl")
            (
                js::import("./runtime/adl", "declResolver"),
                js::import("./runtime/adl", "ScopedDecl"),
            )
        }
    };
    let gened = "/* @generated from adl */";
    // modules.sort_by(|a, b| a.name.cmp(&b.name));
    let mut keys: Vec<String> = modules.iter().map(|m| m.name.clone()).collect();
    // let m_map: HashMap<String,&Module1> = modules.iter().map(|m| (m.name.clone(),*m)).collect();
    keys.sort();
    quote_in! { *t =>
    $gened
    $(register (adlr2))
    $(register (adlr1))
    $(for m in m_imports => $(register (m)))


    export const ADL: { [key: string]: ScopedDecl } = {
      $(for m in keys => ...$(m.replace(".", "_")),$['\r'])
    };

    export const RESOLVER = declResolver(ADL);
    }

    Ok(())
}

pub fn write_runtime(rt_opts: &TsWriteRuntime) -> anyhow::Result<()> {
    let mut writer = TreeWriter::new(PathBuf::from(&rt_opts.output_dir), None)?;
    gen_runtime(rt_opts.strip_first, &rt_opts.ts_style, &mut writer)?;
    Ok(())
}

fn gen_runtime(
    strip_first: bool,
    // rt_gen_opts: &TsGenRuntime,
    ts_style: &TsStyle,
    writer: &mut TreeWriter,
) -> anyhow::Result<()> {
    println!("Writing Runtime to file system ...");
    let re = Regex::new(r"\$TSEXT").unwrap();
    let re2 = Regex::new(r"\$TSB64IMPORT").unwrap();
    for rt_name in Asset::iter() {
        // println!("  '{}'", rt_name);
        let mut file_path = PathBuf::new();
        if !strip_first {
            file_path.push("./runtime");
        }
        // file_path.push(&rt_gen_opts.runtime_dir);
        file_path.push(rt_name.as_ref());
        let dir_path = file_path.parent().unwrap();
        std::fs::create_dir_all(dir_path)?;

        log::info!("writing {}", file_path.display());

        let data = Asset::get(rt_name.as_ref()).unwrap();
        let content = data.data.as_ref();
        match ts_style {
            TsStyle::Tsc => {
                let content = re.replace_all(content, "".as_bytes());
                let content = re2.replace(&content, TSC_B64);
                let x = content.deref();
                let y = String::from_utf8(x.to_vec())?;
                writer.write(file_path.as_path(), y)?;
                // std::fs::write(file_path, content)
                //     .map_err(|e| anyhow!("error writing runtime file {}", e.to_string()))?;
            }
            TsStyle::Deno => {
                let content = re.replace_all(content, ".ts".as_bytes());
                let content = re2.replace(&content, DENO_B64);
                let x = content.deref();
                let y = String::from_utf8(x.to_vec())?;
                writer.write(file_path.as_path(), y)?;
            }
        }
    }
    Ok(())
}
