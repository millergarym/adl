use std::collections::HashMap;

use anyhow::anyhow;
use serde_json::Value;

use crate::adlgen::sys::adlast2::{
    Decl, DeclType, Field, Module, PrimitiveType, TypeExpr, TypeRef,
};
use crate::processing::resolver::Resolver;
use genco::prelude::*;

const DQ: &str = "\"";
const OC: &str = "{";
const CC: &str = "}";
const OSB: &str = "[";
const CSB: &str = "]";

pub struct ResolverModule<'a> {
    pub module: &'a Module<TypeExpr<TypeRef>>,
    pub resolver: &'a Resolver,
}

pub struct TsDefaultValue<'a> {
    pub ctx: &'a ResolverModule<'a>,
    pub decl: &'a Decl<TypeExpr<TypeRef>>,
    pub type_map: &'a HashMap<String, &'a TypeExpr<TypeRef>>,
    // pub field: &'a Field<TypeExpr<TypeRef>>,
}

// pub struct TsDefaultTypeExpr<'a> {
//     pub ctx: &'a ResolverModule<'a>,

//     pub decl: &'a Decl<TypeExpr<TypeRef>>,
//     pub type_map: HashMap<String, &'a TypeExpr<TypeRef>>,
//     // pub type_expr: &'a TypeExpr<TypeRef>,
// }

impl TsDefaultValue<'_> {
    fn create_err(
        &self,
        typename: &str,
        decl_name: &String,
        f_name: &String,
        val: &Value,
    ) -> anyhow::Result<()> {
        let x = serde_json::to_string(val).unwrap();
        return Err(anyhow!(
            "default value does not match. Expected '{}' for {}.{}::{} received '{}'",
            typename,
            self.ctx.module.name,
            decl_name,
            f_name,
            x
        ));
    }
    fn create_err_msg(
        &self,
        typename: &str,
        decl_name: &String,
        f_name: &String,
        val: &Value,
        msg: String,
    ) -> anyhow::Result<()> {
        let x = serde_json::to_string(val).unwrap();
        return Err(anyhow!(
            "default value does not match. Expected '{}' for {}.{}::{} received '{}'\n{}",
            typename,
            self.ctx.module.name,
            decl_name,
            f_name,
            x,
            msg,
        ));
    }
    fn create_err_mismatch_type_params(
        &self,
        decl_name: &String,
        got: usize,
        expected: usize,
    ) -> anyhow::Result<()> {
        return Err(anyhow!(
            "Mismatched number of type parameters. Type constructur for '{}.{}' expected {} arguments, but was passed {}",
            self.ctx.module.name,
            decl_name,
            got,
            expected,
        ));
    }
    fn create_err_missing_val(&self, decl_name: &String, f_name: &String) -> anyhow::Result<()> {
        return Err(anyhow!(
            "Missing value or default value. {}.{}::{}",
            self.ctx.module.name,
            decl_name,
            f_name,
        ));
    }
    fn create_err_missing_type_param(
        &self,
        decl_name: &String,
        f_name: &String,
        tp: &String,
    ) -> anyhow::Result<()> {
        return Err(anyhow!(
            "Missing type param. {}.{}::{}<{}>",
            self.ctx.module.name,
            decl_name,
            f_name,
            tp,
        ));
    }
    fn find_local_decl(&self, name: &String) -> &Decl<TypeExpr<TypeRef>> {
        self.ctx
            .module
            .decls
            .iter()
            .find(|decl| decl.name == *name)
            .unwrap()
    }

    pub fn gen_default_value(
        &self,
        t: &mut Tokens<JavaScript>,
        field: &Field<TypeExpr<TypeRef>>,
        val: Option<&Value>,
    ) -> anyhow::Result<()> {
        let val1 = match val {
            Some(v) => v,
            None => match &field.default.0 {
                Some(v) => v,
                None => {
                    return self.create_err_missing_val(&self.decl.name, &field.name);
                }
            },
        };
        self.gen_type_expr(t, &field.name, &field.type_expr, val1)?;
        Ok(())
    }

    pub fn gen_type_expr(
        &self,
        t: &mut Tokens<JavaScript>,
        f_name: &String,
        type_expr: &TypeExpr<TypeRef>,
        val: &Value,
    ) -> anyhow::Result<()> {
        match &type_expr.type_ref {
            TypeRef::ScopedName(d) => {
                quote_in! { *t => {} };
            }
            TypeRef::LocalName(d) => {
                let decl = self.find_local_decl(&d);
                let type_params = crate::utils::ast::get_type_params(decl);

                if type_expr.parameters.len() != type_params.len() {
                    return self.create_err_mismatch_type_params(
                        &decl.name,
                        type_params.len(),
                        type_expr.parameters.len(),
                    );
                }
                let mut type_map: HashMap<String, &TypeExpr<TypeRef>> = HashMap::new();
                for (i, tp) in type_params.iter().enumerate() {
                    let te_p = type_expr.parameters.get(i).unwrap();
                    type_map.insert(tp.to_string(), te_p);
                }
                let tsgen_te = TsDefaultValue {
                    ctx: self.ctx,
                    type_map: &type_map,
                    decl,
                };
                quote_in! { *t => $OC };
                tsgen_te.gen_type_ref(t, &f_name, val)?;
                quote_in! { *t => $CC };
                // self.gen_default_decl(t, name, decl, f, d, val)?;
            }
            TypeRef::Primitive(d) => {
                self.gen_primitive(t, &f_name, d, val, &type_expr.parameters)?;
            }
            TypeRef::TypeParam(d) => {
                if let Some(te) = self.type_map.get(d) {
                    self.gen_type_expr(t, f_name, te, val)?;
                } else {
                    return self.create_err_missing_type_param(&self.decl.name, &f_name, d);
                }
            }
        }
        Ok(())
    }

    fn gen_type_ref(
        &self,
        t: &mut Tokens<JavaScript>,
        f_name: &String,
        val: &Value,
    ) -> anyhow::Result<()> {
        match &self.decl.r#type {
            DeclType::Struct(ty) => {
                if let Some(obj) = val.as_object() {
                    let mut rest = false;
                    for f0 in &ty.fields {
                        let dvg = TsDefaultValue {
                            ctx: self.ctx,
                            decl: self.decl,
                            type_map: &self.type_map,
                        };
                        if rest {
                            quote_in! { *t => ,$[' '] };
                        } else {
                            rest = true;
                        }
                        quote_in! { *t => $(&f0.serialized_name) :$[' ']}
                        dvg.gen_default_value(t, &f0, obj.get(&f0.serialized_name))?;
                    }
                    // let x = serde_json::to_string(val).unwrap();
                    // quote_in! { *t => $x };
                } else {
                    return self.create_err("object", &self.decl.name, f_name, val);
                }
            }
            DeclType::Union(_ty) => {
                if let Some(obj) = val.as_object() {
                    let y: Vec<&String> = obj.keys().collect();
                    let x = serde_json::to_string(&obj.get(y[0])).unwrap();
                    quote_in! { *t => { kind: $DQ$(y[0])$DQ, value: $x} };
                } else {
                    return self.create_err("object", &self.decl.name, f_name, val);
                }
            }
            DeclType::Type(_ty) => todo!(),
            DeclType::Newtype(_ty) => todo!(),
        };
        Ok(())
    }

    fn gen_primitive(
        &self,
        t: &mut Tokens<JavaScript>,
        f_name: &String,
        type_: &PrimitiveType,
        val: &Value,
        type_params: &Vec<TypeExpr<TypeRef>>,
    ) -> anyhow::Result<()> {
        match type_ {
            PrimitiveType::Void => {
                if let Some(_) = val.as_null() {
                    quote_in! { *t => null }
                } else {
                    return self.create_err("Void", &self.decl.name, f_name, val);
                }
            }
            PrimitiveType::Bool => {
                if let Some(v) = val.as_bool() {
                    if v {
                        quote_in! { *t => true }
                    } else {
                        quote_in! { *t => false }
                    }
                } else {
                    return self.create_err("Bool", &self.decl.name, f_name, val);
                }
            }
            PrimitiveType::Int8 => {
                if let Some(v) = val.as_i64() {
                    // TODO check bounds
                    quote_in! { *t => $(v) }
                } else {
                    return self.create_err("Int8", &self.decl.name, f_name, val);
                }
            }
            PrimitiveType::Int16 => {
                if let Some(v) = val.as_i64() {
                    // TODO check bounds
                    quote_in! { *t => $(v) }
                } else {
                    return self.create_err("Int16", &self.decl.name, f_name, val);
                }
            }
            PrimitiveType::Int32 => {
                if let Some(v) = val.as_i64() {
                    // TODO check bounds
                    quote_in! { *t => $(v) }
                } else {
                    return self.create_err("Int32", &self.decl.name, f_name, val);
                }
            }
            PrimitiveType::Int64 => {
                if let Some(v) = val.as_i64() {
                    // TODO check bounds
                    quote_in! { *t => $(v) }
                } else {
                    return self.create_err("Int64", &self.decl.name, f_name, val);
                }
            }
            PrimitiveType::Word8 => {
                if let Some(v) = val.as_u64() {
                    // TODO check bounds
                    quote_in! { *t => $(v) }
                } else {
                    return self.create_err("Word8", &self.decl.name, f_name, val);
                }
            }
            PrimitiveType::Word16 => {
                if let Some(v) = val.as_u64() {
                    // TODO check bounds
                    quote_in! { *t => $(v) }
                } else {
                    return self.create_err("Word8", &self.decl.name, f_name, val);
                }
            }
            PrimitiveType::Word32 => {
                if let Some(v) = val.as_u64() {
                    // TODO check bounds
                    quote_in! { *t => $(v) }
                } else {
                    return self.create_err("Word8", &self.decl.name, f_name, val);
                }
            }
            PrimitiveType::Word64 => {
                if let Some(v) = val.as_u64() {
                    // TODO check bounds
                    quote_in! { *t => $(v) }
                } else {
                    return self.create_err("Word8", &self.decl.name, f_name, val);
                }
            }
            PrimitiveType::Float => {
                if let Some(v) = val.as_f64() {
                    // Is there a standard JS float format?
                    let v = format!("{}", v);
                    quote_in! { *t => $(v) }
                } else {
                    return self.create_err("Float", &self.decl.name, f_name, val);
                }
            }
            PrimitiveType::Double => {
                if let Some(v) = val.as_f64() {
                    let v = format!("{}", v);
                    quote_in! { *t => $(v) }
                } else {
                    return self.create_err("Float", &self.decl.name, f_name, val);
                }
            }
            PrimitiveType::Json => match serde_json::to_string(val) {
                Ok(x) => {
                    quote_in! { *t => $x };
                }
                Err(e) => {
                    return self.create_err_msg(
                        "Json",
                        &self.decl.name,
                        f_name,
                        val,
                        e.to_string(),
                    );
                }
            },
            PrimitiveType::ByteVector => {
                // duplicating existing adlc, but it's not quite correct.
                // needs to import b64

                // todo check valid base64 encoding
                if let Some(v) = val.as_str() {
                    quote_in! { *t => b64.toByteArray($DQ$(v)$DQ) }
                } else {
                    return self.create_err("Bytes", &self.decl.name, f_name, val);
                }
            }
            PrimitiveType::String => {
                if let Some(v) = val.as_str() {
                    quote_in! { *t => $DQ$(v)$DQ }
                } else {
                    return self.create_err("String", &self.decl.name, f_name, val);
                }
            }
            PrimitiveType::Vector => {
                if let Some(vs) = val.as_array() {
                    quote_in! { *t =>  $OSB }
                    let mut rest = false;
                    for v in vs {
                        if rest {
                            quote_in! { *t => ,$[' '] };
                        } else {
                            rest = true;
                        }
                        // we have already checked that type_params.len() == 1
                        self.gen_type_expr(t, f_name, type_params.get(0).unwrap(), v)?;
                    }
                    quote_in! { *t =>  $CSB }
                } else {
                    return self.create_err("String", &self.decl.name, f_name, val);
                }
            }
            PrimitiveType::StringMap => {
                if let Some(vs) = val.as_object() {
                    quote_in! { *t =>  $OC }
                    let mut rest = false;
                    // serde use a BTreeMap, same a Haskell.
                    // If `preserve_order` feature is used for serde an index map would and the order would be different.
                    for (k, v) in vs {
                        if rest {
                            quote_in! { *t => ,$[' '] };
                        } else {
                            rest = true;
                        }
                        quote_in! { *t => $DQ$(k)$DQ :$[' '] };
                        // we have already checked that type_params.len() == 1
                        self.gen_type_expr(t, f_name, type_params.get(0).unwrap(), v)?;
                    }
                    quote_in! { *t =>  $CC }
                } else {
                    return self.create_err("String", &self.decl.name, f_name, val);
                }
            }
            PrimitiveType::Nullable => {
                if val.is_null() {
                    quote_in! { *t => null }
                } else {
                    self.gen_type_expr(t, f_name, type_params.get(0).unwrap(), val)?;
                }
            }
            PrimitiveType::TypeToken => {
                // This is not quite correct but it is never used since 'makeXXX' are not created if there is a tokentype field (the check is transitive).
                // In the Haskell adlc the check isn't transitive.
                // The actual output should be;
                // - for primatives "ADL.texprInt64()"
                // - for localnames "texprXXX()"
                // - for scopednames ...
                if let Some(_) = val.as_null() {
                    quote_in! { *t => null }
                } else {
                    return self.create_err("TypeToken", &self.decl.name, f_name, val);
                }
            }
        }

        Ok(())
    }
}