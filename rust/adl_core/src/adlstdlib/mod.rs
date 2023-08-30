use std::{borrow::Cow, path::PathBuf};

use anyhow::anyhow;

use rust_embed::{EmbeddedFile, RustEmbed};
use serde::Deserialize;

use crate::{
    adlgen::{adlc::{workspace::{EmbeddedBundle}, bundle::AdlBundle}, sys::adlast2::ModuleName},
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Stdlib {
    Sys,
    Adlc,
}

#[derive(RustEmbed)]
#[folder = "../../adl/stdlib/"]
struct StdlibAsset;

#[derive(RustEmbed)]
#[folder = "../../adl/tools/adlc/"]
struct AdlcAsset;

pub fn get_file_names(em: EmbeddedBundle) -> Vec<PathBuf> {
    match em {
        EmbeddedBundle::Sys => StdlibAsset::iter().map(String::from).map(PathBuf::from).collect(),
        EmbeddedBundle::Adlc => AdlcAsset::iter().map(String::from).map(PathBuf::from).collect(),
    }
}

// fn str_from_cow(name: Cow<str>) -> PathBuf {
//     PathBuf::from(name.to_string());
//     path.to_str().unwrap().to_string()
// }

pub fn get_adl_bundle(em: &EmbeddedBundle) -> Option<Cow<'static, [u8]>> {
    let d = match em {
        EmbeddedBundle::Sys => StdlibAsset::get("adl.bundle.json"),
        EmbeddedBundle::Adlc => AdlcAsset::get("adl.bundle.json"),
    };
    d.map(|d| d.data)
}

pub fn get_stdlib(em: &EmbeddedBundle, mn: &ModuleName, ext: &str) -> Option<(AdlBundle, Cow<'static, [u8]>)> {
    let mut fname = mn.replace(".", "/");
    fname.push_str(".adl");
    if ext != "" {
        fname.push_str("-");
        fname.push_str(ext);
    }
    let pkg_content = get_adl_bundle(em);
    let content = std::str::from_utf8(pkg_content.as_ref().unwrap()).unwrap();
    let de = &mut serde_json::Deserializer::from_str(&content);
    let pkg = AdlBundle::deserialize(de).unwrap();
    let get = match em {
        EmbeddedBundle::Sys => StdlibAsset::get,
        EmbeddedBundle::Adlc => AdlcAsset::get,
    };
    if let Some(f) = get(fname.as_str()) {
        return Some((pkg, f.data));
    };
    None
}

pub fn dump_stdlib(lib: Stdlib, outputdir: PathBuf) -> Result<(), anyhow::Error> {
    std::fs::create_dir_all(&outputdir)
        .map_err(|_| anyhow!("can't create output dir '{:?}'", outputdir))?;
    match lib {
        Stdlib::Sys => {
            for name in StdlibAsset::iter() {
                fun_name(outputdir.clone(), name, StdlibAsset::get)?;
            }
        }
        Stdlib::Adlc => {
            for name in AdlcAsset::iter() {
                fun_name(outputdir.clone(), name, StdlibAsset::get)?;
            }
        }
    };
    Ok(())
}

type Getter = fn(&str) -> Option<EmbeddedFile>;

fn fun_name(mut path: PathBuf, name: Cow<'_, str>, get: Getter) -> Result<(), anyhow::Error> {
    path.push(name.as_ref());
    Ok(if let Some(data) = get(name.as_ref()) {
        std::fs::create_dir_all(path.parent().unwrap())
            .map_err(|_| anyhow!("can't create output dir for '{:?}'", &path))?;
        std::fs::write(&path, data.data.as_ref())
            .map_err(|s| anyhow!("can't write file '{:?}' error {}", &path, s))?;
    } else {
        return Err(anyhow!("could get the contents for {}", name));
    })
}
