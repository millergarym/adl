use super::TsOpts;

use std::cmp::Ordering;
use std::collections::HashMap;

use anyhow::anyhow;
use genco::prelude::js::Import as JsImport;
use genco::tokens::{Item, ItemStr};

use crate::adlgen::sys::adlast2::{
    Annotations, Decl, DeclType, Field, Ident, Module, NewType, PrimitiveType, ScopedName, Struct,
    TypeDef, TypeExpr, TypeRef, Union,
};
use crate::adlrt::custom::sys::types::maybe::Maybe;
use crate::parser::docstring_scoped_name;
use crate::processing::loader::loader_from_search_paths;
use crate::processing::resolver::Resolver;
use genco::fmt::{self, Indentation};
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
                _map: HashMap::new(),
                toks: &mut tokens,
            };
            mgen.gen_module(m)?;
            let stdout = std::io::stdout();
            let mut w = fmt::IoWriter::new(stdout.lock());
            let fmt = fmt::Config::from_lang::<JavaScript>();
            let fmt = fmt::Config::with_indentation(fmt, Indentation::Space(2));

            let config = js::Config::default();
            // let config = js::Config{
            //     ..Default::default()
            // };
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

struct TsScopedDeclGenVisitor<'a> {
    module_name: &'a String,
    toks: &'a mut Tokens<JavaScript>,
}

impl TsScopedDeclGenVisitor<'_> {
    fn lit(&mut self, s: &'static str) {
        self.toks.append(Item::Literal(ItemStr::Static(s)));
    }
}

impl TsScopedDeclGenVisitor<'_> {
    fn visit_annotations(&mut self, d: &Annotations) {
        let mut keys: Vec<&ScopedName> = d.0.keys().collect();
        keys.sort_by(|a, b| {
            if &a.module_name == &b.module_name {
                if a.name == b.name {
                    Ordering::Equal
                } else if a.name > b.name {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            } else if a.module_name > b.module_name {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        });
        quote_in! { self.toks =>  "annotations":$("[") };
        keys.iter().fold(false, |rest, key| {
            if **key == crate::parser::docstring_scoped_name() {
                return rest;
            }
            if rest {
                self.lit(",");
            }
            self.lit("{");
            let jv = &serde_json::to_string(d.0.get(key).unwrap()).unwrap();
            quote_in! { self.toks => "value":$jv,}
            quote_in! { self.toks => "key": }
            self.lit("{");
            quote_in! { self.toks => "moduleName":$DQ$(&key.module_name)$DQ,"name":$DQ$(&key.name)$DQ }
            self.lit("}");
            self.lit("}");
            return true;
        });
        quote_in! { self.toks =>  $("]") };
    }

    fn visit_decl(&mut self, d: &Decl<TypeExpr<TypeRef>>) {
        self.lit("{");
        self.visit_annotations(&d.annotations);
        self.lit(",");
        self.visit_decl_type(&d.r#type);
        self.lit(",");
        self.visit_decl_name(&d.name);
        self.lit(",");
        quote_in! { self.toks => "version":{"kind":"nothing"}};
        self.lit("}");
    }
    fn visit_decl_name(&mut self, n: &String) {
        quote_in! { self.toks =>  "name":$("\"")$n$("\"")};
    }
    fn visit_decl_type(&mut self, r#type: &DeclType<TypeExpr<TypeRef>>) {
        self.lit("\"type_\":{");
        match r#type {
            DeclType::Struct(dt) => self.visit_struct(dt),
            DeclType::Union(dt) => self.visit_union(dt),
            DeclType::Newtype(dt) => self.visit_newtype(dt),
            DeclType::Type(dt) => self.visit_typealias(dt),
        }
        self.lit("}");
    }
    fn visit_struct(&mut self, dt: &Struct<TypeExpr<TypeRef>>) {
        quote_in! { self.toks => "kind":"struct_","value":$("{") }
        self.visit_type_params(&dt.type_params);
        self.lit(",");
        self.lit("\"fields\":[");
        dt.fields.iter().fold(false, |rest, f| {
            if rest {
                self.lit(",");
            }
            self.visit_field(f);
            return true;
        });
        self.lit("]");
        self.lit("}");
    }
    fn visit_union(&mut self, dt: &Union<TypeExpr<TypeRef>>) {
        quote_in! { self.toks => "kind":"union_","value":$("{") }
        self.visit_type_params(&dt.type_params);
        self.lit(",");
        self.lit("\"fields\":[");
        dt.fields.iter().fold(false, |rest, f| {
            if rest {
                self.lit(",");
            }
            self.visit_field(f);
            return true;
        });
        self.lit("]");
        self.lit("}");
    }
    fn visit_newtype(&mut self, _dt: &NewType<TypeExpr<TypeRef>>) {
        // TODO
        println!("newtype: ");
    }
    fn visit_typealias(&mut self, _dt: &TypeDef<TypeExpr<TypeRef>>) {
        // TODO
        println!("type: ");
    }
    fn visit_type_params(&mut self, tps: &Vec<Ident>) {
        quote_in! { self.toks => "typeParams":[$(for tp in tps join (,) => $tp)]}
    }
    fn visit_field(&mut self, f: &Field<TypeExpr<TypeRef>>) {
        self.lit("{");
        self.visit_annotations(&f.annotations);
        self.lit(",");
        quote_in! { self.toks =>  "serializedName":$("\"")$(&f.serialized_name)$("\""), }
        self.visit_default(&f.default);
        self.lit(",");
        quote_in! { self.toks =>  "name":$("\"")$(&f.name)$("\"")};
        self.lit(",");
        quote_in! { self.toks =>  "typeExpr":}
        self.visit_type_expr(&f.type_expr);
        self.lit("}");
    }
    fn visit_default(&mut self, f: &Maybe<serde_json::Value>) {
        quote_in! { self.toks =>  "default":$("{")};
        match f {
            Maybe(None) => {
                quote_in! { self.toks =>  "kind":"nothing"}
            }
            Maybe(Some(v)) => {
                let jv = &serde_json::to_string(&v).unwrap();
                quote_in! { self.toks =>  "kind":"just","value":$(jv)};
            }
        }
        quote_in! { self.toks =>  $("}")};
    }
    fn visit_type_expr(&mut self, te: &TypeExpr<TypeRef>) {
        quote_in! { self.toks =>  $("{")}
        self.visit_type_ref(&te.type_ref);
        quote_in! { self.toks => ,"parameters":$("[")}
        te.parameters.iter().fold(false, |rest, p| {
            if rest {
                self.lit(",");
            }
            self.visit_type_expr(p);
            return true;
        });
        self.lit("]");
        self.lit("}");
    }
    fn visit_type_ref(&mut self, te: &TypeRef) {
        quote_in! { self.toks => "typeRef":$("{")}
        match te {
            TypeRef::ScopedName(n) => {
                quote_in! { self.toks =>  "kind":"reference","value":{"moduleName":$("\""):$(&n.module_name)$("\""),"name":$("\"")$(&n.name)$("\"")}};
            }
            TypeRef::LocalName(n) => {
                quote_in! { self.toks =>  "kind":"reference","value":{"moduleName":$("\""):$(self.module_name)$("\""),"name":$("\"")$(n)$("\"")}};
            }
            TypeRef::Primitive(n) => {
                let p = crate::processing::primitives::str_from_prim(n.clone());
                // let p = &serde_json::to_string(n).unwrap();
                quote_in! { self.toks =>  "kind":"primitive","value":$DQ$p$DQ};
            }
            TypeRef::TypeParam(n) => {
                quote_in! { self.toks =>  "kind":"typeParam","value":$("\"")$n$("\"")};
            }
        }
        self.lit("}");
    }
}

struct TsGenVisitor<'a> {
    toks: &'a mut Tokens<JavaScript>,
    adlr: JsImport,
    _map: HashMap<String, JsImport>,
}

impl TsGenVisitor<'_> {
    fn lit(&mut self, s: &'static str) {
        self.toks.append(Item::Literal(ItemStr::Static(s)));
    }
}

impl TsGenVisitor<'_> {
    fn gen_doc_comment(&mut self, annotations: &Annotations) {
        if let Some(ds) = annotations.0.get(&docstring_scoped_name()) {
            self.lit("/**\n");
            for c in ds.as_array().unwrap().iter() {
                if let Ok(x) = serde_json::to_string(&c.clone()) {
                    // TODO should this be trimmed? or should the output be "*$y" ie no space
                    let y = x[1..x.len() - 1].trim();
                    quote_in! {self.toks => $[' ']* $(y)$['\r']};
                }
            }
            self.lit(" */\n");
        }
    }
    fn gen_module(&mut self, m: &Module<TypeExpr<TypeRef>>) -> anyhow::Result<()> {
        quote_in! { self.toks =>
            $("/* @generated from adl module") $(m.name.clone()) $("*/")
            $['\n']
        };
        for decl in m.decls.iter() {
            self.gen_doc_comment(&decl.annotations);
            let r = match &decl.r#type {
                DeclType::Struct(d) => self.gen_struct(d, DeclPayload(decl)),
                DeclType::Union(d) => self.gen_union(d, DeclPayload(decl)),
                DeclType::Newtype(d) => self.gen_newtype(d),
                DeclType::Type(d) => self.gen_type(d),
            };
            // Generation AST holder
            let name = &decl.name;
            let name_up = capitalize_first(name);
            let mname = m.name.clone();
            quote_in! { self.toks =>
                $['\n']
                const $(name_up)_AST : $(&self.adlr).ScopedDecl =
                  {"moduleName":$("\"")$(mname.clone())$("\""),"decl":$(ref tok => {
                    let mut sdg = TsScopedDeclGenVisitor{module_name: &mname.clone(), toks: tok};
                    sdg.visit_decl(decl);
                  })};

                export const sn$(name): $(&self.adlr).ScopedName = {moduleName:$("\"")$mname$("\""), name:$("\"")$name$("\"")};

                export function texpr$(name)(): ADL.ATypeExpr<$(name)> {
                    return {value : {typeRef : {kind: "reference", value : sn$(name)}, parameters : []}};
                }
                $['\n']
            }

            if let Err(_) = r {
                return r;
            }
        }
        self.lit("export const _AST_MAP: { [key: string]: ADL.ScopedDecl } = {\n");
        m.decls.iter().fold(false, |rest, decl| {
            if rest {
                self.lit(",\n")
            }
            self.lit("  ");
            quote_in! { self.toks =>
                $("\"")$(m.name.clone()).$(&decl.name)$("\"") : $(capitalize_first(&decl.name))_AST
            };
            true
        });
        self.lit("\n};");
        Ok(())
    }
}

const OC: &str = "{";
const CC: &str = "}";
const DQ: &str = "\"";
const SP: &str = " ";

impl TsGenVisitor<'_> {
    fn gen_struct(
        &mut self,
        m: &Struct<TypeExpr<TypeRef>>,
        payload: DeclPayload,
    ) -> anyhow::Result<()> {
        let (decl, name) = (&payload.0, &payload.0.name);
        let name_up = &capitalize_first(name);
        quote_in! { self.toks =>
            $("// struct")$['\n']
        }
        self.gen_doc_comment(&decl.annotations);
        quote_in! { self.toks =>
            export interface $(name_up) $OC$['\r']
        }

        for f in m.fields.iter() {
            self.gen_doc_comment(&f.annotations);
            let rt = rust_type(&f.type_expr).map_err(|s| anyhow!(s))?;
            quote_in! { self.toks =>
                $SP$SP$(&f.name): $(rt);$['\r']
            }
        }
        quote_in! { self.toks =>
            $CC$['\r']$['\n']
        }
        quote_in! { self.toks =>
            export function make$(name_up)(
              input: {
                $(ref tok => struct_field_make_input(tok, &m.fields)?)
              }
            ): $(name_up) {
              return {
                $(ref tok => struct_field_make_return(tok, &m.fields))
              };
            }
        }
        Ok(())
    }
}

fn struct_field_make_input(
    toks: &mut Tokens<JavaScript>,
    fs: &Vec<Field<TypeExpr<TypeRef>>>,
) -> anyhow::Result<()> {
    for f in fs {
        let rt = rust_type(&f.type_expr).map_err(|s| anyhow!(s))?;
        quote_in! { *toks =>
          $(&f.name): $(rt),$['\r']
        }
    }
    Ok(())
}

fn struct_field_make_return(toks: &mut Tokens<JavaScript>, fs: &Vec<Field<TypeExpr<TypeRef>>>) {
    for f in fs {
        quote_in! { *toks =>
          $(&f.name): input.$(&f.name),$['\r']
        }
    }
}

struct DeclPayload<'a>(&'a Decl<TypeExpr<TypeRef>>);

impl TsGenVisitor<'_> {
    fn gen_union(
        &mut self,
        m: &Union<TypeExpr<TypeRef>>,
        payload: DeclPayload,
    ) -> anyhow::Result<()> {
        let name = &payload.0.name;
        self.lit("// union \n");
        let is_enum = m
            .fields
            .iter()
            .find(|f| match &f.type_expr.type_ref {
                TypeRef::Primitive(p) => match p {
                    PrimitiveType::Void => false,
                    _ => true,
                },
                _ => true,
            })
            .is_none();
        if !is_enum {
            let mut bnames_up = vec![];
            let mut opts = vec![];
            for b in m.fields.iter() {
                self.gen_doc_comment(&b.annotations);
                let bname = b.name.clone();
                let bname_up = capitalize_first(&b.name);
                bnames_up.push(bname_up.clone());
                let rtype = rust_type(&b.type_expr).map_err(|s| anyhow!(s))?;
                opts.push((bname.clone(), rtype.clone()));
                quote_in! { self.toks =>
                    export interface $(name)_$(bname_up) {
                        kind: $("'")$(bname)$("'");
                        value: $(rtype);
                    }$['\r']
                }
            }
            quote_in! { self.toks =>
                $['\n']
                export type $name = $(for n in bnames_up join ( | ) => $(name)_$n);

                export interface $(name)Opts {
                  $(for opt in opts => $(opt.0): $(opt.1);$['\r'])
                }$['\n']

                export function make$(name)<K extends keyof $(name)Opts>(kind: K, value: $(name)Opts[K]) { return {kind, value}; }$['\n']
            }
        } else {
            let b_names: Vec<&String> = m.fields.iter().map(|f| &f.name).collect();
            let b_len = b_names.len();
            let b1 = if b_len > 0 { b_names[0] } else { "" };
            quote_in! { self.toks =>
                $['\n']
                export type $name = $(for n in b_names join ( | ) => $("'")$(n)$("'"));
                $['\r']
            }
            // TODO not sure what this is for -- duplicating existing ts
            if b_len == 1 {
                quote_in! { self.toks => export const values$name : $name[] = [$("'")$(b1)$("'")];$['\r'] }
            }
        }
        Ok(())
    }

    fn gen_newtype(&mut self, _m: &NewType<TypeExpr<TypeRef>>) -> anyhow::Result<()> {
        quote_in! { self.toks =>
            $("// newtype")
        }
        Ok(())
    }

    fn gen_type(&mut self, _m: &TypeDef<TypeExpr<TypeRef>>) -> anyhow::Result<()> {
        quote_in! { self.toks =>
            $("// type")
        }
        Ok(())
    }
}

pub fn capitalize_first(input: &String) -> String {
    let mut c = input.chars();
    match c.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().to_string() + &String::from(&input[1..]),
    }
}

fn rust_type(te: &TypeExpr<TypeRef>) -> Result<String, String> {
    match &te.type_ref {
        TypeRef::ScopedName(_n) => todo!(),
        TypeRef::LocalName(_n) => todo!(),
        TypeRef::Primitive(n) => tstype_from_prim(n, &te.parameters),
        TypeRef::TypeParam(_n) => todo!(),
    }
}

fn tstype_from_prim(
    prim: &PrimitiveType,
    params: &Vec<TypeExpr<TypeRef>>,
) -> Result<String, String> {
    match prim {
        PrimitiveType::Void => Ok("null".to_string()),
        PrimitiveType::Bool => Ok("boolean".to_string()),
        PrimitiveType::Int8 => Ok("number".to_string()),
        PrimitiveType::Int16 => Ok("number".to_string()),
        PrimitiveType::Int32 => Ok("number".to_string()),
        PrimitiveType::Int64 => Ok("number".to_string()),
        PrimitiveType::Word8 => Ok("number".to_string()),
        PrimitiveType::Word16 => Ok("number".to_string()),
        PrimitiveType::Word32 => Ok("number".to_string()),
        PrimitiveType::Word64 => Ok("number".to_string()),
        PrimitiveType::Float => Ok("number".to_string()),
        PrimitiveType::Double => Ok("number".to_string()),
        PrimitiveType::Json => Ok("{}|null".to_string()),
        PrimitiveType::ByteVector => Ok("Uint8Array".to_string()),
        PrimitiveType::String => Ok("string".to_string()),
        _ => {
            if params.len() != 1 {
                return Err(format!( "Primitive parameterized type require 1 and only one param. Type {:?} provided with {}", prim, params.len() ))
            }
            let param_type = rust_type(&params[0])?;
            match prim {
                PrimitiveType::Vector => {
                    return Ok(format!("{}[]", param_type));
                }
                PrimitiveType::StringMap => Ok(format!("{}[key: string]: {}{}", "{", param_type, "}")),
                PrimitiveType::Nullable => Ok(format!("({}|null)", param_type)),
                PrimitiveType::TypeToken => Ok(format!("ADL.ATypeExpr<{}>", param_type)),
                _ => Err(format!( "unknown primitive {:?}", prim ))
            }
        }
    }
}
