use super::TsOpts;

use std::collections::HashMap;
use std::error::Error;

use anyhow::anyhow;
use genco::prelude::js::Import as JsImport;
use genco::tokens::{Item, ItemStr};

use crate::adlgen::sys::adlast2::{
    Decl, DeclType, Field, Ident, Import, Module, NewType, PrimitiveType, ScopedName, Struct,
    TypeDef, TypeExpr, TypeRef, Union,
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
        if let Some(m) = resolver.get_module(mn) {
            // TODO sys.annotations::SerializedName needs to be embedded
            let mut tokens = js::Tokens::new();
            let mut mgen = TsGenVisitor {
                adlr: js::import("./runtime/adl", "ADL").into_wildcard(),
                map: HashMap::new(),
                tokens: &mut tokens,
            };
            mgen.gen_module(m);

            let stdout = std::io::stdout();
            let mut w = fmt::IoWriter::new(stdout.lock());
            let fmt = fmt::Config::from_lang::<JavaScript>();
            let config = js::Config::default();
            tokens.format_file(&mut w.as_formatter(&fmt), &config)?;
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

trait Visitor {
    fn visit_Module(&mut self, m: &Module<TypeExpr<TypeRef>>);
    fn visit_annotations(&mut self, d: &Map<ScopedName, serde_json::Value>);
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
    fn visit_type_ref(&mut self, te: &TypeRef);
}

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
        self.tokens
            .append(Item::Literal(ItemStr::Static("\"type_\":{")));
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
        self.tokens
            .append(Item::Literal(ItemStr::Static("\"fields\":[")));
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
            Maybe(None) => {
                quote_in! { self.tokens =>  "kind":"nothing"}
            }
            Maybe(Some(v)) => {
                let jv = &serde_json::to_string(&v).unwrap();
                quote_in! { self.tokens =>  "kind":"just","value":$(jv)};
            }
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
            }
            TypeRef::LocalName(n) => {
                quote_in! { self.tokens =>  "kind":"reference","value":{"moduleName":$("\""):$(self.module_name)$("\""),"name":$("\"")$(n)$("\"")}};
            }
            TypeRef::Primitive(n) => {
                let p = &serde_json::to_string(n).unwrap();
                quote_in! { self.tokens =>  "kind":"primitive","value":$p};
            }
            TypeRef::TypeParam(n) => {
                quote_in! { self.tokens =>  "kind":"typeParam","value":$("\"")$n$("\"")};
            }
        }
        self.tokens.append(Item::Literal(ItemStr::Static("}")));
    }
}

struct TsGenVisitor<'a> {
    tokens: &'a mut Tokens<JavaScript>,
    adlr: JsImport,
    map: HashMap<String, JsImport>,
}

impl TsGenVisitor<'_> {
    fn gen_module(&mut self, m: &Module<TypeExpr<TypeRef>>) -> anyhow::Result<()> {
        quote_in! { self.tokens =>
            $("/* @generated from adl module") $(m.name.clone()) $("*/")
            $['\n']
        };
        for decl in m.decls.iter() {
            let r = match &decl.r#type {
                DeclType::Struct(d) => Ok(()),
                DeclType::Union(d) => self.gen_union(d, GenUnionPayload(decl, m.name.clone())),
                DeclType::Newtype(d) => Ok(()),
                DeclType::Type(d) => Ok(()),
            };
            if let Err(_) = r {
                return r;
            }
        }
        self.tokens.append(Item::Literal(ItemStr::Static(
            "export const _AST_MAP: { [key: string]: ADL.ScopedDecl } = {\n",
        )));
        for decl in m.decls.iter() {
            quote_in! { self.tokens =>
                $[' ']$[' ']$("\"")$(m.name.clone()).$(&decl.name)$("\"") : $(&decl.name)_AST,$['\r']
            }
        }
        self.tokens.append(Item::Literal(ItemStr::Static("}")));
        Ok(())
    }
}

impl TsGenVisitor<'_> {
    fn gen_struct(&mut self, name: &String, m: &Struct<TypeExpr<TypeRef>>) -> anyhow::Result<()> {
        quote_in! { self.tokens =>
            $("// struct")
            // export type $name;

            // const $(name)_AST : $(&imports.adlr).ScopedDecl
            // export const sn$(name): $(&imports.adlr).ScopedName = {moduleName:"test5", name:"U1"};
            $['\n']
        }
        Ok(())
    }
}

struct GenUnionPayload<'a>(&'a Decl<TypeExpr<TypeRef>>, String);

impl TsGenVisitor<'_> {
    fn gen_union(
        &mut self,
        m: &Union<TypeExpr<TypeRef>>,
        payload: GenUnionPayload,
    ) -> anyhow::Result<()> {
        let (decl, name, mname) = (payload.0, &payload.0.name, payload.1);
        self.tokens
            .append(Item::Literal(ItemStr::Static("// union \n")));
        let is_enum = m
            .fields
            .iter()
            .find(|f| match &f.type_expr.type_ref {
                TypeRef::ScopedName(_) => true,
                TypeRef::LocalName(_) => true,
                TypeRef::TypeParam(_) => true,
                TypeRef::Primitive(p) => match p {
                    PrimitiveType::Void => false,
                    _ => true,
                },
            })
            .is_none();
        if !is_enum {
            let mut bnames_up = vec![];
            let mut opts = vec![];
            for b in m.fields.iter() {
                let bname = b.name.clone();
                let bname_up = b.name.to_uppercase();
                bnames_up.push(bname_up.clone());
                let rtype = rust_type(&b.type_expr);
                opts.push((bname.clone(), rtype.clone()));
                quote_in! { self.tokens =>
                    export interface $(name)_$(bname_up) {
                        kind: $("'")$(bname)$("'");
                        value: $(rtype);
                    }$['\r']
                }
            }
            quote_in! { self.tokens =>
                $['\n']
                export type $name = $(for n in bnames_up join ( | ) => $(name)_$n);

                export interface $(name)Opts {
                  $(for opt in opts => $(opt.0): $(opt.1);$['\r'])
                }$['\n']

                export function make$(name)<K extends keyof $(name)Opts>(kind: K, value: $(name)Opts[K]) { return {kind, value}; }$['\n']
            }
        } else { // enum
            let b_names: Vec<&String> = m.fields.iter().map(|f| &f.name).collect();
            let b_len = b_names.len();
            let b1 = if b_len > 0 { b_names[0] } else { "" };
            quote_in! { self.tokens =>
                $['\n']
                export type $name = $(for n in b_names join ( | ) => $("'")$(n)$("'"));
                $['\r']
            }
            // TODO not sure what this is for -- duplicating existing ts
            if b_len == 1 {
                quote_in! { self.tokens => export const values$name : $name[] = [$("'")$(b1)$("'")];$['\r'] }
            }
        }

        quote_in! { self.tokens =>
            $['\n']
            const $(name)_AST : $(&self.adlr).ScopedDecl =
              {"moduleName":$("\"")$(mname.clone())$("\""),"decl":$(ref tok => {
                let mut sdg = TsScopedDeclGenVisitor{module_name: &mname.clone(), tokens: tok};
                sdg.visit_decl(decl);
              })}

            export const sn$(name): $(&self.adlr).ScopedName = {moduleName:$("\"")$mname$("\""), name:$("\"")$name$("\"")};

            export function texpr$(name)(): ADL.ATypeExpr<$(name)> {
                return {value : {typeRef : {kind: "reference", value : sn$(name)}, parameters : []}};
            }
            $['\n']
        }
        Ok(())
    }

    fn gen_newtype(
        &mut self,
        tokens: &mut Tokens<JavaScript>,
        name: &String,
        m: &NewType<TypeExpr<TypeRef>>,
    ) -> anyhow::Result<()> {
        quote_in! { *tokens =>
            $("// newtype")
        }
        Ok(())
    }

    fn gen_type(
        &mut self,
        tokens: &mut Tokens<JavaScript>,
        name: &String,
        m: &TypeDef<TypeExpr<TypeRef>>,
    ) -> anyhow::Result<()> {
        quote_in! { *tokens =>
            $("// type")
        }
        Ok(())
    }
}

fn rust_type<TE>(te: &TypeExpr<TE>) -> String {
    return "number".to_string();
}
