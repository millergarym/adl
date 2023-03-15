use super::TsOpts;

use anyhow::anyhow;

use crate::adlgen::sys::adlast2::{Module, TypeExpr, TypeRef};
use crate::processing::loader::loader_from_search_paths;
use crate::processing::resolver::{Resolver};
use genco::fmt;
use genco::prelude::*;

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
    for mn in &opts.modules {
        // TODO why is the resolver by the filename not the module name
        if let Some(m) = resolver.get_module(mn) {
            gen_module(m);
            // println!("// gen {}", m.name);
        }
    }
    // let modules: Vec<&Module1> = resolver
    //     .get_module_names()
    //     .into_iter()
    //     .map(|mn| resolver.get_module(&mn).unwrap())
    //     .collect();
    // println!("{}", serde_json::to_string_pretty(&modules).unwrap());
    Ok(())
}


fn gen_module(m: &Module<TypeExpr<TypeRef>>) -> anyhow::Result<()> {
    let mut tokens = js::Tokens::new();
    let adlr = &js::import("./runtime/adl", "ADL").into_wildcard();

    // println!("// gen {}", m.name);

    quote_in! { tokens =>
        $("/* @generated from adl module") $(m.name.clone()) $("*/")
        $['\n']
    };

    for (name, decl) in m.decls.iter() {
        quote_in! { tokens =>
            export type $name;

            const $(name)_AST : $adlr.ScopedDecl
            export const sn$(name): $adlr.ScopedName = {moduleName:"test5", name:"U1"};
            $['\n']
        }
    }

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt = fmt::Config::from_lang::<JavaScript>();
    let config = js::Config::default();

    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
    Ok(())
}


fn gencoexample() -> anyhow::Result<()> {
    let adlr = &js::import("./runtime/adl", "ADL").into_wildcard();
    let react = &js::import("react", "React").into_default();
    let display = &js::import("./Display", "Display").into_default();
    let button_panel = &js::import("./ButtonPanel", "ButtonPanel").into_default();
    let calculate = &js::import("../logic/calculate", "calculate").into_default();

    let tokens = quote! {
$adlr

export type U1 = 'v';
export const valuesU1 : U1[] = ['v'];

const U1_AST : ADL.ScopedDecl =
  {"moduleName":"test5","decl":{"annotations":[],"type_":{"kind":"union_","value":{"typeParams":[],"fields":[{"annotations":[],"serializedName":"v","default":{"kind":"nothing"},"name":"v","typeExpr":{"typeRef":{"kind":"primitive","value":"Void"},"parameters":[]}}]}},"name":"U1","version":{"kind":"nothing"}}};

export const snU1: ADL.ScopedName = {moduleName:"test5", name:"U1"};

export function texprU1(): ADL.ATypeExpr<U1> {
  return {value : {typeRef : {kind: "reference", value : snU1}, parameters : []}};
}

export interface U2_V {
  kind: 'v';
  value: number;
}

export const _AST_MAP: { [key: string]: ADL.ScopedDecl } = {
  "test5.U1" : U1_AST,
  "test5.U2" : U2_AST,
  "test5.U3" : U3_AST,
  "test5.S1" : S1_AST,
  "test5.U4" : U4_AST,
  "test5.U5" : U5_AST,
  "test5.U6" : U6_AST,
  "test5.U7" : U7_AST,
  "test5.U8" : U8_AST,
  "test5.U9" : U9_AST,
  "test5.S" : S_AST,
  "test5.List" : List_AST,
  "test5.Cell" : Cell_AST,
  "test5.U10" : U10_AST,
  "test5.S10" : S10_AST,
  "test5.U11" : U11_AST,
  "test5.S11" : S11_AST
};

        export type U2 = U2_V;

        export default class App extends $react.Component {
            state = {
                total: null,
                next: null,
                operation: null,
            };

            handleClick = buttonName => {
                this.setState($calculate(this.state, buttonName));
            };

            render() {
                return (
                    <div className="component-app">
                        <$display value={this.state.next || this.state.total || "0"} />
                        <$button_panel clickHandler={this.handleClick} />
                    </div>
                );
            }
        }
    };

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt = fmt::Config::from_lang::<JavaScript>();
    let config = js::Config::default();

    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
    Ok(())
}