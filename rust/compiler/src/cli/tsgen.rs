use super::TsOpts;

use std::collections::HashMap;
use std::error::Error;

use anyhow::anyhow;
use genco::prelude::js::Import as JsImport;
use genco::tokens::{Item, ItemStr};

use crate::adlgen::sys::adlast2::{
    Decl, DeclType, Import, Module, NewType, Struct, TypeDef, TypeExpr, TypeRef, Union, Ident, Field, ScopedName,
};
use crate::adlrt::custom::sys::types::map::Map;
use crate::adlrt::custom::sys::types::maybe::Maybe;
use crate::processing::loader::loader_from_search_paths;
use crate::processing::resolver::Resolver;
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


// trait VisitModuleName<I,O> {
//     type Option<In>;
//     type Result<Out,Error>;

//     fn visit_module_name(&mut self, i: Self::Option<I>, n: &String) -> Self::Result<O,String>;
// }

// struct DefaultVisiter {}

// struct Unit{}

// impl VisitModuleName for DefaultVisiter {
//     type Option<In> = Option<_>;
//     type Result<Out,Error>;
//     fn visit_module_name(&mut self, i: None, n: &String) -> None {
//         println!("!!moduleName: {}", n);
//         return Unit{};
//     }

// }

trait Visitor {
    fn visit_Module (&mut self, m: &Module<TypeExpr<TypeRef>>);
    fn visit_module_name(&mut self, n: &String);
    fn visit_import(&mut self, i: &Import);
    fn visit_decl(&mut self, d: &Decl<TypeExpr<TypeRef>>);
    fn visit_decl_name(&mut self, n: &String);
    fn visit_decl_type(&mut self, r#type: &DeclType<TypeExpr<TypeRef>>);
    fn visit_struct(&mut self, dt: &Struct<TypeExpr<TypeRef>>);
    fn visit_union(&mut self, dt: &Union<TypeExpr<TypeRef>>);
    fn visit_newtype(&mut self, dt: &NewType<TypeExpr<TypeRef>>);
    fn visit_typealias(&mut self, dt: &TypeDef<TypeExpr<TypeRef>>);
    fn visit_type_params(&mut self, tps: &Vec<Ident>);
    fn visit_field(&mut self, f: &Field<TypeExpr<TypeRef>>);
    fn visit_default(&mut self, f: &Maybe<serde_json::Value>);
    fn visit_type_expr(&mut self, te: &TypeExpr<TypeRef>);
    fn visit_type_ref(&mut self, te: &TypeRef);}

struct TsScopedDeclGenVisitor<'a> {
    module_name: &'a String,
    tokens: &'a mut Tokens<JavaScript>,
}

impl TsScopedDeclGenVisitor<'_> {
    fn visit_annotations(&mut self, d: &Map<ScopedName, serde_json::Value>) {
        quote_in! { self.tokens =>  "annotations":$("[") };
        // d.0.keys()
        quote_in! { self.tokens =>  $("]") };
    }
    fn visit_decl(&mut self, d: &Decl<TypeExpr<TypeRef>>) {
        self.tokens.append(Item::Literal(ItemStr::Static("{")));
        self.visit_annotations(&d.annotations);
        self.visit_decl_type(&d.r#type);
        self.tokens.append(Item::Literal(ItemStr::Static(",")));
        self.visit_decl_name(&d.name);
        self.tokens.append(Item::Literal(ItemStr::Static(",")));
        quote_in! { self.tokens => "version":{"kind":"nothing"}};
        self.tokens.append(Item::Literal(ItemStr::Static("}")));
    }
    fn visit_decl_name(&mut self, n: &String) {
        quote_in! { self.tokens =>  "name":$("\"")$n$("\"")};
    }
    fn visit_decl_type(&mut self, r#type: &DeclType<TypeExpr<TypeRef>>) {
        self.tokens.append(Item::Literal(ItemStr::Static("\"type_\":{")));
        match r#type {
            DeclType::Struct(dt) => self.visit_struct(dt),
            DeclType::Union(dt) => self.visit_union(dt),
            DeclType::Newtype(dt) => self.visit_newtype(dt),
            DeclType::Type(dt) => self.visit_typealias(dt),
        }
        self.tokens.append(Item::Literal(ItemStr::Static("}")));
    }
    fn visit_struct(&mut self, dt: &Struct<TypeExpr<TypeRef>>) {
        println!("struct: ");
    }
    fn visit_union(&mut self, dt: &Union<TypeExpr<TypeRef>>) {
        quote_in! { self.tokens => "kind":"union_","value":$("{") }
        self.visit_type_params(&dt.type_params);
        self.tokens.append(Item::Literal(ItemStr::Static(",")));
        self.tokens.append(Item::Literal(ItemStr::Static("\"fields\":[")));
        let mut it = dt.fields.iter().peekable();
        while let Some(f) = it.next() {
            self.visit_field(f);
            if it.peek().is_some() {
                self.tokens.append(Item::Literal(ItemStr::Static(",")));
            }
        }
        self.tokens.append(Item::Literal(ItemStr::Static("]")));
        self.tokens.append(Item::Literal(ItemStr::Static("}")));

    }
    fn visit_newtype(&mut self, dt: &NewType<TypeExpr<TypeRef>>) {
        println!("newtype: ");
    }
    fn visit_typealias(&mut self, dt: &TypeDef<TypeExpr<TypeRef>>) {
        println!("type: ");
    }
    fn visit_type_params(&mut self, tps: &Vec<Ident>) {
        quote_in! { self.tokens => "typeParams":[$(for tp in tps join (,) => $tp)]}
    }
    fn visit_field(&mut self, f: &Field<TypeExpr<TypeRef>>) {
        self.tokens.append(Item::Literal(ItemStr::Static("{")));
        quote_in! { self.tokens =>  "annotations":$("[")}
        // f.annotations
        quote_in! { self.tokens => $("]"),};
        quote_in! { self.tokens =>  "serializedName":$("\"")$(&f.serialized_name)$("\""), }
        self.visit_default(&f.default);
        self.tokens.append(Item::Literal(ItemStr::Static(",")));
        self.visit_type_expr(&f.type_expr);
        self.tokens.append(Item::Literal(ItemStr::Static(",")));
        quote_in! { self.tokens =>  "name":$("\"")$(&f.name)$("\"")};
        self.tokens.append(Item::Literal(ItemStr::Static("}")));
    }
    fn visit_default(&mut self, f: &Maybe<serde_json::Value>) {
        quote_in! { self.tokens =>  "default":$("{")};
        match f {
            Maybe(None) => { quote_in! { self.tokens =>  "kind":"nothing"} },
            Maybe(Some(v)) => { 
                let jv = &serde_json::to_string(&v).unwrap();
                quote_in! { self.tokens =>  "kind":"just","value":$(jv)};
            },
        }
        quote_in! { self.tokens =>  $("}")};
    }
    fn visit_type_expr(&mut self, te: &TypeExpr<TypeRef>) {
        quote_in! { self.tokens =>  "typeExpr":$("{")}
        self.visit_type_ref(&te.type_ref);
        quote_in! { self.tokens => ,"parameters":$("[")}
        let mut it = te.parameters.iter().peekable();
        while let Some(p) = it.next() {
            self.visit_type_expr(p);
            if it.peek().is_some() {
                self.tokens.append(Item::Literal(ItemStr::Static(",")));
            }
        }
        self.tokens.append(Item::Literal(ItemStr::Static("]")));
        self.tokens.append(Item::Literal(ItemStr::Static("}")));
    }
    fn visit_type_ref(&mut self, te: &TypeRef) {
        quote_in! { self.tokens => "typeExpr":$("{")}
        match te {
            TypeRef::ScopedName(n) => {
                quote_in! { self.tokens =>  "kind":"reference","value":{"moduleName":$("\""):$(&n.module_name)$("\""),"name":$("\"")$(&n.name)$("\"")}};
            },
            TypeRef::LocalName(n) => {
                quote_in! { self.tokens =>  "kind":"reference","value":{"moduleName":$("\""):$(self.module_name)$("\""),"name":$("\"")$(n)$("\"")}};
            },
            TypeRef::Primitive(n) => {
                let p = &serde_json::to_string(n).unwrap();
                quote_in! { self.tokens =>  "kind":"primitive","value":$p};
            },
            TypeRef::TypeParam(n) => {
                quote_in! { self.tokens =>  "kind":"typeParam","value":$("\"")$n$("\"")};
            },
        }
        self.tokens.append(Item::Literal(ItemStr::Static("}")));
    }
}

struct Imports {
    adlr: JsImport,
    map: HashMap<String, JsImport>,
}

fn gen_module(m: &Module<TypeExpr<TypeRef>>) -> anyhow::Result<()> {
    let mut tokens = js::Tokens::new();
    let imports = Imports {
        adlr: js::import("./runtime/adl", "ADL").into_wildcard(),
        map: HashMap::new(),
    };
    // let adlr = &js::import("./runtime/adl", "ADL").into_wildcard();

    // println!("// gen {}", m.name);
    // let mut x = DefaultVisiter{};
    // x.visit_module_name(Unit{}, &m.name);
    // module_walker(Box::new(DefaultAdlAst2Visitor {}), m);
    // module_walker(&mut MyVisitor(DefaultVisitor {}), m);
    // module_walker(&mut DefaultVisitor {}, m);

    quote_in! { tokens =>
        $("/* @generated from adl module") $(m.name.clone()) $("*/")
        $['\n']
    };

    for decl in m.decls.iter() {
        // let scopedDecl = ScopedDecl::new(m.name.clone(), decl);
        // let scopedDecl = ScopedDecl {
        //     module_name: m.name.clone(),
        //     decl: *decl,
        // };
        let r = match &decl.r#type {
            DeclType::Struct(d) => Ok(()),
            DeclType::Union(d) => gen_union(&mut tokens, &imports, m.name.clone(), decl, d),
            DeclType::Newtype(d) => Ok(()),
            DeclType::Type(d) => Ok(()),
        };
        if let Err(_) = r {
            return r;
        }
    }

    tokens.append(Item::Literal(ItemStr::Static(
        "export const _AST_MAP: { [key: string]: ADL.ScopedDecl } = {\n",
    )));
    for decl in m.decls.iter() {
        quote_in! { tokens =>
            $[' ']$[' ']$("\"")$(m.name.clone()).$(&decl.name)$("\"") : $(&decl.name)_AST,$['\r']
        }
    }
    tokens.append(Item::Literal(ItemStr::Static("}")));

    let stdout = std::io::stdout();
    let mut w = fmt::IoWriter::new(stdout.lock());

    let fmt = fmt::Config::from_lang::<JavaScript>();
    let config = js::Config::default();

    tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
    Ok(())
}

fn gen_struct(
    tokens: &mut Tokens<JavaScript>,
    imports: &Imports,
    name: &String,
    m: &Struct<TypeExpr<TypeRef>>,
) -> anyhow::Result<()> {
    quote_in! { *tokens =>
        $("// struct")
        // export type $name;

        // const $(name)_AST : $(&imports.adlr).ScopedDecl
        // export const sn$(name): $(&imports.adlr).ScopedName = {moduleName:"test5", name:"U1"};
        $['\n']
    }
    Ok(())
}

fn rust_type<TE>(te: &TypeExpr<TE>) -> String {
    return "number".to_string();
}

fn gen_union(
    tokens: &mut Tokens<JavaScript>,
    imports: &Imports,
    mname: String,
    decl: &Decl<TypeExpr<TypeRef>>,
    m: &Union<TypeExpr<TypeRef>>,
) -> anyhow::Result<()> {
    // let scopedDecl = ScopedDecl::new(mname.clone(), *decl);
    // let scopedDecl = ScopedDecl {
    //     module_name: mname.clone(),
    //     decl: *(decl.clone()),
    // };
    // TODO this is wireformat need TS format
    // TODO sys.annotations::SerializedName needs to be embedded
    let ast_decl = serde_json::to_string(decl).unwrap();
    let name = &decl.name;
    tokens.append(Item::Literal(ItemStr::Static("// union \n")));
    // let ast = serde_json::to_string(&scopedDecl).unwrap();
    let mut bnames_up = vec![];
    let mut opts = vec![];
    for b in m.fields.iter() {
        let bname = b.name.clone();
        let bname_up = b.name.to_uppercase();
        bnames_up.push(bname_up.clone());
        let rtype = rust_type(&b.type_expr);
        opts.push((bname.clone(), rtype.clone()));
        quote_in! { *tokens =>
            export interface $(name)_$(bname_up) {
                kind: $("'")$(bname)$("'");
                value: $(rtype);
            }$['\r']
        }
    }
    quote_in! { *tokens =>
        $['\n']
        export type $name = $(for n in bnames_up join ( | ) => $(name)_$n);

        export interface $(name)Opts {
          $(for opt in opts => $(opt.0): $(opt.1);$['\r'])
        }$['\n']

        export function make$(name)<K extends keyof $(name)Opts>(kind: K, value: $(name)Opts[K]) { return {kind, value}; }

        const $(name)_AST : $(&imports.adlr).ScopedDecl =
          {"moduleName":$("\"")$(mname.clone())$("\""),"decl":$(ref tok => {
            let mut sdg = TsScopedDeclGenVisitor{module_name: &mname.clone(), tokens: tok};
            sdg.visit_decl(decl);
          })}

        export const sn$(name): $(&imports.adlr).ScopedName = {moduleName:$("\"")$mname$("\""), name:$("\"")$name$("\"")};

        export function texpr$(name)(): ADL.ATypeExpr<$(name)> {
            return {value : {typeRef : {kind: "reference", value : sn$(name)}, parameters : []}};
        }
        $['\n']
    }
    Ok(())
}

fn gen_newtype(
    tokens: &mut Tokens<JavaScript>,
    imports: &Imports,
    name: &String,
    m: &NewType<TypeExpr<TypeRef>>,
) -> anyhow::Result<()> {
    quote_in! { *tokens =>
        $("// newtype")
    }
    Ok(())
}

fn gen_type(
    tokens: &mut Tokens<JavaScript>,
    imports: &Imports,
    name: &String,
    m: &TypeDef<TypeExpr<TypeRef>>,
) -> anyhow::Result<()> {
    quote_in! { *tokens =>
        $("// type")
    }
    Ok(())
}
