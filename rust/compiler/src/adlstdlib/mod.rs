use std::path::PathBuf;
use std::borrow::Cow;

use anyhow::anyhow;

use rust_embed::RustEmbed;

use crate::adlgen::sys::adlast2::ModuleName;

#[derive(RustEmbed)]
#[folder = "../../adl/stdlib/"]
struct Asset;

pub fn std_modules() -> Vec<ModuleName> {
    let mut mods = vec![];
    for name in Asset::iter() {
        let mut path = PathBuf::from(name.to_string());
        if let Some(e) = path.extension() {
            if e == "adl" {
                path.set_extension("");
                mods.push(path.to_str().unwrap().replace("/", "."));
            }
        }
    };
    mods
}

pub fn get_stdlib(mn: &ModuleName, ext: &str) -> Option<Cow<'static, [u8]>> {
    let mut fname = mn.replace(".", "/");
    fname.push_str(".adl");
    if ext != "" {
        fname.push_str("-");
        fname.push_str(ext);
    }
    if let Some(f) = Asset::get(fname.as_str()) {
        return Some(f.data);
    }
    None
}

pub(crate) fn dump(opts: &crate::cli::DumpStdlibOpts) -> Result<(), anyhow::Error> {
    std::fs::create_dir_all(&opts.outputdir).map_err(|_| anyhow!("can't create output dir '{:?}'", opts.outputdir))?;
    for name in Asset::iter() {
        let mut path = opts.outputdir.clone();
        path.push(name.as_ref());
        if let Some(data) = Asset::get(name.as_ref()) {
            std::fs::create_dir_all(path.parent().unwrap()).map_err(|_| anyhow!("can't create output dir for '{:?}'", &path))?;
            std::fs::write(&path, data.data.as_ref()).map_err(|s| anyhow!("can't write file '{:?}' error {}", &path, s))?;
        } else {
            return Err(anyhow!("could get the contents for {}", name));
        }
    }
    Ok(())
}