use serde_json::Value;

use crate::adlgen::sys::adlast2::{
    DeclType, Field, Module, PrimitiveType, ScopedName, TypeExpr, TypeRef,
};
use crate::processing::resolver::Resolver;
use genco::prelude::*;

const DQ: &str = "\"";

pub struct TsDefaultValue<'a> {
    pub module: &'a Module<TypeExpr<TypeRef>>,
    pub resolver: &'a Resolver,
}

impl TsDefaultValue<'_> {
    pub fn gen_default_value(
        &self,
        t: &mut Tokens<JavaScript>,
        f: &Field<TypeExpr<TypeRef>>,
    ) -> anyhow::Result<()> {
        if let Some(val) = &f.default.0 {
            match &f.type_expr.type_ref {
                TypeRef::ScopedName(d) => self.gen_default_scope_name(t, d, val)?,
                TypeRef::LocalName(d) => self.gen_default_local_name(t, d, val)?,
                TypeRef::Primitive(d) => self.gen_default_primitive(t, d, val)?,
                TypeRef::TypeParam(d) => self.gen_default_type_param(t, d, val)?,
            }
        } else {
            // find out how to unwrap the default.0
            todo!()
        }
        Ok(())
    }

    fn gen_default_scope_name(
        &self,
        t: &mut Tokens<JavaScript>,
        d: &ScopedName,
        val: &Value,
    ) -> anyhow::Result<()> {
        // let decl = self.resolver.get_decl(d).unwrap();
        quote_in! { *t => {} };
        Ok(())
    }

    fn gen_default_local_name(
        &self,
        t: &mut Tokens<JavaScript>,
        d: &String,
        val: &Value,
    ) -> anyhow::Result<()> {
        let decl = self
            .module
            .decls
            .iter()
            .find(|decl| decl.name == *d)
            .unwrap();
        match &decl.r#type {
            DeclType::Struct(ty) => {
                let x = serde_json::to_string(val).unwrap();
                quote_in! { *t => $x };
            }
            DeclType::Union(ty) => {
                let x = val.as_object().unwrap();
                let y: Vec<&String> = x.keys().collect();
                let x = serde_json::to_string(&x.get(y[0])).unwrap();
                quote_in! { *t => { kind: $DQ$(y[0])$DQ, value: $x} };
            }
            DeclType::Type(ty) => todo!(),
            DeclType::Newtype(ty) => todo!(),
        };
        Ok(())
    }

    fn gen_default_primitive(
        &self,
        t: &mut Tokens<JavaScript>,
        d: &PrimitiveType,
        val: &Value,
    ) -> anyhow::Result<()> {
        let x = serde_json::to_string(d).unwrap();
        quote_in! { *t => $x };
        Ok(())
    }

    fn gen_default_type_param(
        &self,
        t: &mut Tokens<JavaScript>,
        d: &String,
        val: &Value,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
