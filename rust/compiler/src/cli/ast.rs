use std::collections::BTreeMap;

use super::AstOpts;

use anyhow::anyhow;

use crate::processing::loader::loader_from_search_paths;
use crate::processing::resolver::{Module1, Resolver};

pub fn ast(opts: &AstOpts) -> anyhow::Result<()> {
    let loader = loader_from_search_paths(&opts.search.path);
    let mut resolver = Resolver::new(loader);
    for m in &opts.modules {
        let r = resolver.add_module(m);
        match r {
            Ok(()) => (),
            Err(e) => return Err(anyhow!("Failed to load module {}: {:?}", m, e)),
        }
    }
    // let modules: Vec<&Module1> = resolver
    //     .get_module_names()
    //     .into_iter()
    //     .map(|mn| resolver.get_module(&mn).unwrap())
    //     .collect();

    let mut json_mod: BTreeMap<String,&Module1> = BTreeMap::new();
    resolver.get_module_names().iter().for_each(|mn| {
        let m = resolver.get_module(&mn).unwrap();
        json_mod.insert(mn.to_string(), m.into());
    });

    // let mut buf = Vec::new();
    // let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    // let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    
    // obj.serialize(&mut ser).unwrap();
    // println!("{}", String::from_utf8(buf).unwrap());

    println!("{}", serde_json::to_string_pretty(&json_mod).unwrap());
    Ok(())
}
