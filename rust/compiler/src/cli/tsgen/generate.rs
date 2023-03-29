use convert_case::{Case, Casing};
use std::collections::HashMap;

use anyhow::anyhow;

use genco::prelude::js::Import as JsImport;
use genco::prelude::*;
use genco::tokens::{Item, ItemStr};

use crate::adlgen::sys::adlast2::{
    Annotations, Decl, DeclType, Module, NewType, PrimitiveType, ScopedName, Struct, TypeDef,
    TypeExpr, TypeRef, Union,
};
use crate::parser::docstring_scoped_name;
use crate::processing::resolver::Resolver;

const SP: &str = " ";
const SQ: &str = "'";

pub struct TsGenVisitor<'a> {
    pub module: &'a Module<TypeExpr<TypeRef>>,
    pub resolver: &'a Resolver,
    pub adlr: JsImport,
    pub map: &'a mut HashMap<ScopedName, (String, JsImport)>,
    pub opts: &'a crate::cli::TsOpts,
}

// struct DeclPayload<'a>(&'a Decl<TypeExpr<TypeRef>>);
struct DeclPayload<'a> {
    decl: &'a Decl<TypeExpr<TypeRef>>,
    mname: &'a String,
}

struct RttiPayload<'a> {
    mname: String,
    type_params: &'a Vec<String>,
}

impl TsGenVisitor<'_> {
    pub fn gen_module(
        &mut self,
        t: &mut Tokens<JavaScript>,
        m: &Module<TypeExpr<TypeRef>>,
    ) -> anyhow::Result<()> {
        quote_in! { *t =>
            $("/* @generated from adl module") $(m.name.clone()) $("*/")
            $['\n']
        };
        let mname = &m.name;
        for decl in m.decls.iter() {
            self.gen_doc_comment(t, &decl.annotations);
            let payload = DeclPayload {
                decl: decl,
                mname: &mname.clone(),
            };
            let r = match &decl.r#type {
                DeclType::Struct(d) => self.gen_struct(t, d, payload),
                DeclType::Union(d) => self.gen_union(t, d, payload),
                DeclType::Newtype(d) => self.gen_newtype(t, d, payload),
                DeclType::Type(d) => self.gen_type(t, d, payload),
            };
            if let Err(_) = r {
                return r;
            }
        }
        lit(
            t,
            "export const _AST_MAP: { [key: string]: ADL.ScopedDecl } = {\n",
        );
        m.decls.iter().fold(false, |rest, decl| {
            if rest {
                lit(t, ",\n")
            }
            lit(t, "  ");
            quote_in! { *t =>
                $("\"")$(m.name.clone()).$(&decl.name)$("\"") : $(&decl.name)_AST
            };
            true
        });
        lit(t, "\n};");
        for (_, v) in self.map.iter() {
            let imp = &v.1;
            quote_in! {*t => $(register (imp))}
        }
        Ok(())
    }
}

impl TsGenVisitor<'_> {
    fn gen_struct(
        &mut self,
        t: &mut Tokens<JavaScript>,
        m: &Struct<TypeExpr<TypeRef>>,
        payload: DeclPayload<'_>,
    ) -> anyhow::Result<()> {
        let (decl, name) = (payload.decl, &payload.decl.name);
        // let name_up = &title(name);
        self.gen_doc_comment(t, &decl.annotations);
        // let fnames: &Vec<String> = &m.fields.iter().map(|f| f.name.clone()).collect();
        let te_trs: Vec<TypeExpr<TypeRef>> = m.fields.iter().map(|f| f.type_expr.clone()).collect();
        let mut has_make = true;
        let fnames = used_type_params(&te_trs);
        quote_in! { *t =>
            export interface $(name)$(gen_type_params_(&fnames, &m.type_params)) {$['\r']
            $(for f in m.fields.iter() =>
                $(ref t => {
                    self.gen_doc_comment(t, &f.annotations);
                    let rt = self.rust_type(&f.type_expr).map_err(|s| anyhow!(s))?;
                    has_make = has_make && rt.0;
                    quote_in! { *t =>
                        $SP$SP$(&f.name): $(rt.1);$['\r']
                    }
                })
            )}$['\r']$['\n']
        }
        if has_make {
            quote_in! { *t =>
                export function make$(cap_or__(name))$(gen_type_params_(&fnames, &m.type_params))(
                  $(if m.fields.len() == 0 => _)input: {
                    $(for f in &m.fields => $(ref t => {
                        let rt = self.rust_type(&f.type_expr).map_err(|s| anyhow!(s))?;
                        let mut has_default = false;
                        if let Some(_) = f.default.0 {
                            has_default = true;
                        }
                        quote_in! { *t => $(&f.name)$(if has_default => ?): $(rt.1),$['\r']}
                    }))
                  }
                ): $(name)$(gen_type_params_(&fnames, &m.type_params)) {
                  return {
                    $(for f in &m.fields => $(ref t => {
                        if let Some(_) = f.default.0 {
                            quote_in! { *t => $(&f.name): input.$(&f.name) === undefined ?$[' '] }
                            let dvg = &mut crate::cli::tsgen::defaultval::TsDefaultValue{
                                ctx: &crate::cli::tsgen::defaultval:: ResolverModule {
                                    module: self.module,
                                    resolver: self.resolver,
                                },
                                decl: decl,
                                type_map: &HashMap::new(),
                                // depth: Box::new(0),
                            };
                            dvg.gen_default_value(t, &f, None)?;
                            quote_in! { *t => $[' ']: input.$(&f.name),$['\r'] }
                        } else {
                            quote_in! { *t => $(&f.name): input.$(&f.name),$['\r'] }
                        }
                    }))
                  };
                }
            }
        }
        self.gen_rtti(
            t,
            decl,
            &RttiPayload {
                mname: payload.mname.clone(),
                type_params: &m.type_params,
            },
        )?;
        Ok(())
    }
}

impl TsGenVisitor<'_> {
    fn gen_union(
        &mut self,
        t: &mut Tokens<JavaScript>,
        m: &Union<TypeExpr<TypeRef>>,
        payload: DeclPayload<'_>,
    ) -> anyhow::Result<()> {
        let name = &payload.decl.name;
        // lit(t, "// union \n");

        let type_suffix = |name0: &String| {
            if self.opts.capitalize_branch_names_in_types {
                to_title(name0)
            } else {
                name0.clone()
            }
        };

        if !crate::cli::tsgen::defaultval::is_enum(m) {
            let mut opts = vec![];
            for b in m.fields.iter() {
                self.gen_doc_comment(t, &b.annotations);
                let bname = b.name.clone();

                let rtype = self.rust_type(&b.type_expr).map_err(|s| anyhow!(s))?;
                let used = used_type_params(&vec![b.type_expr.clone()]);

                opts.push((bname.clone(), rtype.clone().1));

                let is_void = match &b.type_expr.type_ref {
                    TypeRef::Primitive(p) => match p {
                        PrimitiveType::Void => true,
                        _ => false,
                    },
                    _ => false,
                };
                if is_void {
                    quote_in! { *t =>
                        export interface $(name)_$(type_suffix(&bname.clone()))$(gen_type_params_(&used, &m.type_params)) {
                            kind: $SQ$(bname.clone())$SQ;
                        }$['\r']
                    }
                } else {
                    quote_in! { *t =>
                        export interface $(name)_$(type_suffix(&bname.clone()))$(gen_type_params_(&used, &m.type_params)) {
                            kind: $SQ$(bname.clone())$SQ;
                            value: $(rtype.1.clone());
                        }$['\r']
                    }
                }
            }
            let te_trs: Vec<TypeExpr<TypeRef>> =
                m.fields.iter().map(|f| f.type_expr.clone()).collect();
            let tp_names = used_type_params(&te_trs);
            quote_in! { *t =>
                $['\n']
                export type $name$(gen_type_params_(&tp_names, &m.type_params)) = $(for n in m.fields.iter().map(|b| &b.name) join ( | ) =>
                        $(name)_$(type_suffix(n))$(gen_type_params_(&tp_names, &m.type_params))
                    );

                export interface $(name)Opts$(gen_type_params_(&tp_names, &m.type_params)) {
                  $(for opt in opts => $(opt.0): $(opt.1);$['\r'])
                }$['\n']

                export function make$(name)<$(gen_raw_type_params_(&tp_names, &m.type_params))K extends keyof $(name)Opts$(gen_type_params_(&tp_names, &m.type_params))>(kind: K, value: $(name)Opts$(gen_type_params_(&tp_names, &m.type_params))[K]) { return {kind, value}; }$['\n']
            }
        } else {
            let b_names: Vec<&String> = m.fields.iter().map(|f| &f.name).collect();
            quote_in! { *t =>
                $['\n']
                export type $name = $(for n in &b_names join ( | ) => $SQ$(*n)$SQ);
                $['\r']
            }
            quote_in! { *t => export const values$name : $name[] =$[' ']
                [$(for n in b_names join (, ) => $SQ$(n)$SQ)];$['\r']
            }
        }
        self.gen_rtti(
            t,
            payload.decl,
            &RttiPayload {
                mname: payload.mname.clone(),
                type_params: &m.type_params,
            },
        )?;
        Ok(())
    }

    fn gen_newtype(
        &mut self,
        t: &mut Tokens<JavaScript>,
        m: &NewType<TypeExpr<TypeRef>>,
        payload: DeclPayload<'_>,
    ) -> anyhow::Result<()> {
        let name = &payload.decl.name;
        let rtype = self.rust_type(&m.type_expr).map_err(|s| anyhow!(s))?;

        quote_in! { *t =>
            $("// newtype")$['\n']
        }
        let used = used_type_params(&vec![m.type_expr.clone()]);
        quote_in! { *t =>
            export type $name$(gen_type_params_(&used, &m.type_params)) =  $(rtype.1.clone());
        }
        self.gen_rtti(
            t,
            payload.decl,
            &RttiPayload {
                mname: payload.mname.clone(),
                type_params: &m.type_params,
            },
        )?;
        quote_in! { *t =>
            $['\n']
        }
        Ok(())
    }

    fn gen_type(
        &mut self,
        t: &mut Tokens<JavaScript>,
        m: &TypeDef<TypeExpr<TypeRef>>,
        payload: DeclPayload<'_>,
    ) -> anyhow::Result<()> {
        let name = &payload.decl.name;
        let rtype = self.rust_type(&m.type_expr).map_err(|s| anyhow!(s))?;

        quote_in! { *t =>
            $("// type")$['\n']
        }
        let used = used_type_params(&vec![m.type_expr.clone()]);
        quote_in! { *t =>
            export type $name$(gen_type_params_(&used, &m.type_params)) =  $(rtype.1.clone());
        }
        self.gen_rtti(
            t,
            payload.decl,
            &RttiPayload {
                mname: payload.mname.clone(),
                type_params: &m.type_params,
            },
        )?;
        quote_in! { *t =>
            $['\n']
        }
        Ok(())
    }

    /// returns (has_make_function,ts type)
    fn rust_type(&mut self, te: &TypeExpr<TypeRef>) -> Result<(bool, String), String> {
        match &te.type_ref {
            TypeRef::ScopedName(n) => self.tstype_from_scoped_name(n, &te.parameters),
            TypeRef::LocalName(n) => self.tstype_from_local_name(n, &te.parameters),
            TypeRef::Primitive(n) => self.tstype_from_prim(n, &te.parameters),
            TypeRef::TypeParam(n) => Ok((true, n.clone())),
        }
    }

    fn tstype_from_scoped_name(
        &mut self,
        scoped_name: &ScopedName,
        params: &Vec<TypeExpr<TypeRef>>,
    ) -> Result<(bool, String), String> {
        let imp = self.map.entry(scoped_name.clone()).or_insert_with(|| {
            let path =
                crate::cli::tsgen::utils::rel_import(&self.module.name, &scoped_name.module_name);
            let i_name = scoped_name
                .module_name
                .replace(".", "_")
                .to_case(Case::Snake);
            let import = js::import(path, i_name.clone()).into_wildcard();
            (i_name, import)
        });
        let local_name = format!("{}.{}", imp.0.to_string(), scoped_name.name);
        self.tstype_from_local_name(&local_name, params)
    }

    fn tstype_from_local_name(
        &mut self,
        local_name: &String,
        params: &Vec<TypeExpr<TypeRef>>,
    ) -> Result<(bool, String), String> {
        if params.len() > 0 {
            let mut tperr = vec![];
            let tps: Vec<String> = params
                .iter()
                .filter_map(|p| {
                    let r = self.rust_type(p);
                    match r {
                        Ok((_, t)) => Some(t),
                        Err(e) => {
                            tperr.push(e);
                            return None;
                        }
                    }
                })
                .collect();
            if tperr.len() != 0 {
                let msg = tperr.join("\n\t");
                return Err(format!("Error constructing type param: {}", msg));
            }
            let tpstr = format!("{}<{}>", local_name.clone(), tps.join(", "));
            return Ok((true, tpstr));
        }
        Ok((true, local_name.clone()))
    }

    fn tstype_from_prim(
        &mut self,
        prim: &PrimitiveType,
        params: &Vec<TypeExpr<TypeRef>>,
    ) -> Result<(bool, String), String> {
        match prim {
            PrimitiveType::Void => Ok((true, "null".to_string())),
            PrimitiveType::Bool => Ok((true, "boolean".to_string())),
            PrimitiveType::Int8 => Ok((true, "number".to_string())),
            PrimitiveType::Int16 => Ok((true, "number".to_string())),
            PrimitiveType::Int32 => Ok((true, "number".to_string())),
            PrimitiveType::Int64 => Ok((true, "number".to_string())),
            PrimitiveType::Word8 => Ok((true, "number".to_string())),
            PrimitiveType::Word16 => Ok((true, "number".to_string())),
            PrimitiveType::Word32 => Ok((true, "number".to_string())),
            PrimitiveType::Word64 => Ok((true, "number".to_string())),
            PrimitiveType::Float => Ok((true, "number".to_string())),
            PrimitiveType::Double => Ok((true, "number".to_string())),
            PrimitiveType::Json => Ok((true, "{}|null".to_string())),
            PrimitiveType::ByteVector => Ok((true, "Uint8Array".to_string())),
            PrimitiveType::String => Ok((true, "string".to_string())),
            _ => {
                if params.len() != 1 {
                    return Err(format!( "Primitive parameterized type require 1 and only one param. Type {:?} provided with {}", prim, params.len() ));
                }
                let param_type = self.rust_type(&params[0])?;
                match prim {
                    PrimitiveType::Vector => {
                        return Ok((param_type.0, format!("{}[]", param_type.1)));
                    }
                    PrimitiveType::StringMap => Ok((
                        param_type.0,
                        format!("{}[key: string]: {}{}", "{", param_type.1, "}"),
                    )),
                    PrimitiveType::Nullable => {
                        Ok((param_type.0, format!("({}|null)", param_type.1)))
                    }
                    PrimitiveType::TypeToken => {
                        Ok((false, format!("ADL.ATypeExpr<{}>", param_type.1)))
                    }
                    _ => Err(format!("unknown primitive {:?}", prim)),
                }
            }
        }
    }
}

impl TsGenVisitor<'_> {
    fn gen_rtti(
        &mut self,
        t: &mut Tokens<JavaScript>,
        decl: &Decl<TypeExpr<TypeRef>>,
        payload: &RttiPayload<'_>,
    ) -> anyhow::Result<()> {
        // Generation AST holder
        let name = &decl.name;
        // let name_up = title(name);
        let mname = &payload.mname;
        quote_in! { *t =>
            $['\n']
            const $(name)_AST : $(&self.adlr).ScopedDecl =
              {"moduleName":$("\"")$(mname.clone())$("\""),"decl":$(ref tok => {
                let mut sdg = crate::cli::tsgen::astgen::TsScopedDeclGenVisitor{module_name: &mname.clone(), t: tok};
                sdg.visit_decl(decl);
              })};

            export const sn$(cap_or__(name)): $(&self.adlr).ScopedName = {moduleName:$("\"")$mname$("\""), name:$("\"")$name$("\"")};

            export function texpr$(cap_or__(name))$(gen_type_params(payload.type_params))($(ref t => texpr_args(t, &payload.type_params))): ADL.ATypeExpr<$(name)$(gen_type_params(payload.type_params))> {
                return {value:{typeRef:{kind:"reference",value:sn$(cap_or__(name))},parameters:[$(ref t => texpr_params(t, &payload.type_params))]}};
            }
            $['\n']
        }
        Ok(())
    }
}

impl TsGenVisitor<'_> {
    fn gen_doc_comment(&mut self, t: &mut Tokens<JavaScript>, annotations: &Annotations) {
        if let Some(ds) = annotations.0.get(&docstring_scoped_name()) {
            lit(t, "/**\n");
            for c in ds.as_array().unwrap().iter() {
                if let Ok(x) = serde_json::to_string(&c.clone()) {
                    // TODO should this be trimmed? or should the output be "*$y" ie no space
                    let y = x[1..x.len() - 1].trim();
                    quote_in! { *t => $[' ']* $(y)$['\r'] };
                }
            }
            lit(t, " */\n");
        }
    }
}

fn lit(t: &mut Tokens<JavaScript>, s: &'static str) {
    t.append(Item::Literal(ItemStr::Static(s)));
}

fn gen_type_params<'a>(type_params: &'a Vec<String>) -> impl FormatInto<JavaScript> + 'a {
    quote_fn! {
        $(if type_params.len() > 0 => <$(for tp in type_params join (, ) => $tp)>)
    }
}

fn gen_type_params_<'a>(
    used: &'a Vec<String>,
    type_params: &'a Vec<String>,
) -> impl FormatInto<JavaScript> + 'a {
    quote_fn! {
        $(if type_params.len() > 0 => <$(for tp in type_params join (, ) => $(if !used.contains(tp) => _)$tp)>)
    }
}

fn gen_raw_type_params_<'a>(
    used: &'a Vec<String>,
    type_params: &'a Vec<String>,
) -> impl FormatInto<JavaScript> + 'a {
    quote_fn! {
        $(if type_params.len() > 0 => $(for tp in type_params join (, ) => $(if !used.contains(tp) => _)$tp),$[' '])
    }
}

fn texpr_args<'a>(mut t: &mut Tokens<JavaScript>, type_params: &'a Vec<String>) {
    type_params.iter().fold(false, |rest, p| {
        if rest {
            quote_in! { t => ,$[' '] };
        }
        quote_in! { t => texpr$p : ADL.ATypeExpr<$p> };
        true
    });
}

fn texpr_params<'a>(mut t: &mut Tokens<JavaScript>, type_params: &'a Vec<String>) {
    type_params.iter().fold(false, |rest, p| {
        if rest {
            quote_in! { t => , };
        }
        quote_in! { t => texpr$(p).value };
        true
    });
}

fn used_type_params(te_trs: &Vec<TypeExpr<TypeRef>>) -> Vec<String> {
    let fnames = &mut Vec::new();
    collect_used_type_params(&te_trs, fnames);
    fnames.to_vec()
}

fn collect_used_type_params(te_trs: &Vec<TypeExpr<TypeRef>>, mut fnames: &mut Vec<String>) {
    te_trs.iter().for_each(|te| {
        if let TypeRef::TypeParam(tp) = &te.type_ref {
            fnames.push(tp.clone());
        }
        collect_used_type_params(&te.parameters, &mut fnames);
    })
}

pub fn cap_or__(input: &String) -> String {
    let mut c = input.chars();
    match c.next() {
        None => String::new(),
        Some(first) => {
            if first.is_uppercase() {
                return input.clone();
            } else {
                return "_".to_string() + input;
            }
        }
    }
}

pub fn to_title(input: &String) -> String {
    let mut c = input.chars();
    match c.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().to_string() + &String::from(&input[1..]),
    }
}