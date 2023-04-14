use std::fs;
use std::path::{Path, PathBuf};
use std::env;

pub(crate) fn workspace(opts: &super::GenOpts) -> Result<(), anyhow::Error> {
    let pkg_defs = collect_work_and_pkg(&opts.dir);
    println!("{:?}", pkg_defs);
    Ok(())
}

const ADL_PKG_FILES: &[(&str, PkgDef)] = &[("adl.pkg.json", PkgDef::Pkg), ("adl.work.json", PkgDef::Work)];

#[derive(Debug, Copy, Clone, PartialEq)]
enum PkgDef {
    Pkg,
    Work,
}

fn collect_work_and_pkg(start_dir: &String) -> Vec<(PkgDef,PathBuf)> {
    let mut res = vec![];
    let mut current_dir = PathBuf::from(start_dir);
    
    loop {
        for f in ADL_PKG_FILES {
            let file_path = current_dir.join(f.0);
            if file_path.exists() {
                res.push((f.1, current_dir.clone()));
            }
        }
        if !current_dir.pop() {
            break;
        }
    }
    res
}