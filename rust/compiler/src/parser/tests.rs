
use std::fs;
use std::path::PathBuf;

use crate::utils::ast::{mk_typeexpr0, mk_scoped_name};

use super::*;
use nom::{
    error::{ErrorKind, VerboseError, VerboseErrorKind},
    Err as NomErr,
};

#[test]
fn parse_whitespace() {
  assert_parse_ws(whitespace(inp("x")), "x");
  assert_parse_ws(whitespace(inp(" x")), "x");
  assert_parse_ws(whitespace(inp("\n x")), "x");

  assert_parse_ws(whitespace(inp(" / x")),  "/ x");
  assert_parse_ws(whitespace(inp(" // x")), "");
  assert_parse_ws(whitespace(inp(" // x\ny")), "y");
  assert_parse_ws(whitespace(inp("\n// a comment\n x")), "x");
  assert_parse_ws(whitespace(inp(" /// docstring \ny")), "/// docstring \ny");
}

fn assert_parse_ws<T>(pr: Res<Input, T>, remaining: &str) 
  where T: std::fmt::Debug+PartialEq {
  if let Ok((i, _)) = pr  {
    assert_eq!(i.fragment(), &remaining);
  } else {
    panic!("Unexpected parse failure" );
  }
}


#[test]
fn parse_ident0() {
  assert_parse_eq(ident0(inp("x")), "x");
  assert_parse_eq(ident0(inp("X")), "X");
  assert_parse_eq(ident0(inp("xy_z1")), "xy_z1");
  assert_parse_eq_2(ident0(inp("xyz.")), "xyz", ".");

  assert_eq!(
    super::ident0(inp("")),
    Err(NomErr::Error(VerboseError {
      errors: vec![
          (inp(""), VerboseErrorKind::Nom(ErrorKind::Alpha)),
      ]
    }))
  );

  assert_eq!(
    super::ident0(inp("7")),
    Err(NomErr::Error(VerboseError {
      errors: vec![
          (inp("7"), VerboseErrorKind::Nom(ErrorKind::Alpha)),
      ]
    }))
  );
}

  #[test]
fn parse_module_name() {
  assert_parse_eq(module_name(inp("x")), "x".to_owned());
  assert_parse_eq_2(module_name(inp("x.y.z;")), "x.y.z".to_owned(), ";");
}

#[test]
fn parse_scoped_name() {
  assert_parse_eq(scoped_name(inp("x")), adlast::ScopedName::new("".to_string(), "x".to_string()));
  assert_parse_eq(scoped_name(inp("x.y.z")), adlast::ScopedName::new("x.y".to_string(), "z".to_string()));
}

#[test]
fn parse_import() {
  assert_parse_eq(r#import(inp("import x.y.z")), adlast::Import::ScopedName(mk_scoped_name("x.y", "z")));
  assert_parse_eq(r#import(inp("import x.y.*")), adlast::Import::ModuleName("x.y".to_owned()));
}

#[test]
fn parse_type_expr() {

  assert_parse_eq(
    type_expr(inp("a.X")), 
    mk_typeexpr0(  mk_scoped_name("a", "X"))
  );

  assert_parse_eq(
    type_expr(inp("a.X<y.z.B>")), 
    adlast::TypeExpr{
      type_ref: mk_scoped_name("a", "X"),
      parameters: vec![
        mk_typeexpr0(mk_scoped_name("y.z", "B"))
      ]
    }
  );

  assert_parse_eq(
    type_expr(inp("a.X<y.z.B,C>")), 
    adlast::TypeExpr{
      type_ref: mk_scoped_name("a", "X"),
      parameters: vec![
        mk_typeexpr0( mk_scoped_name("y.z", "B")),
        mk_typeexpr0(mk_scoped_name("", "C")),
      ]
    }
  );
}

#[test]
fn parse_decl() {

  assert_parse_eq(
    decl(inp("struct A { F f1; G f2; }")),
    adlast::Decl{
      name: adlast::Spanned::new("A".to_string(), adlast::Span::new(7, 8)),
      version:  Maybe::nothing(),
      annotations:  Map::new(Vec::new()),
      r#type: adlast::DeclType::Struct(adlast::Struct{
        type_params: Vec::new(),
        fields: vec![
          adlast::Field{
            name: adlast::Spanned::new("f1".to_string(), adlast::Span::new(13, 15)),
            annotations:  Map::new(Vec::new()),
            default:  Maybe::nothing(),
            serialized_name: "f1".to_string(),
            type_expr: mk_typeexpr0(mk_scoped_name("", "F")),
          },
          adlast::Field{
            name: adlast::Spanned::new("f2".to_string(), adlast::Span::new(19, 21)),
            annotations:  Map::new(Vec::new()),
            default: Maybe::nothing(),
            serialized_name: "f2".to_string(),
            type_expr: mk_typeexpr0(mk_scoped_name("", "G")),
          }
        ],
      }),
    },
  )
}

#[test]
fn parse_decl_annotations() {

  assert_parse_eq(
    decl(inp("@X.Z true @Y \"xyzzy\" struct A {}")),
      adlast::Decl{ 
      name: adlast::Spanned::new("A".to_string(), adlast::Span::new(28, 29)),
      version:  Maybe::nothing(),
      annotations:  Map::from_iter(vec![
        (mk_scoped_name("", "Y"), serde_json::Value::String("xyzzy".to_owned())),
        (mk_scoped_name("X", "Z"), serde_json::Value::Bool(true)),
      ]),
      r#type: adlast::DeclType::Struct(adlast::Struct{
        type_params: Vec::new(),
        fields: vec![],
      }),
    },
  )
}

#[test]
fn parse_explicit_annotations() {
  assert_parse_eq(
    explicit_annotation(inp("annotation Bool false")),
    ExplicitAnnotation{
      refr: ExplicitAnnotationRef::Module,
      scoped_name: mk_scoped_name("", "Bool"),
      value: serde_json::Value::from(false),
    },
  );

  assert_parse_eq(
    explicit_annotation(inp("annotation MyStruct Bool false")),
    ExplicitAnnotation{
      refr: ExplicitAnnotationRef::Decl("MyStruct".to_owned()),
      scoped_name: mk_scoped_name("", "Bool"),
      value: serde_json::Value::from(false),
    },
  );

  assert_parse_eq(
    explicit_annotation(inp("annotation MyStruct::f1 Bool false")),
    ExplicitAnnotation{
      refr: ExplicitAnnotationRef::Field(("MyStruct".to_owned(), "f1".to_owned())),
      scoped_name: mk_scoped_name("", "Bool"),
      value: serde_json::Value::from(false),
    },
  );
}

#[test]
fn parse_docstring() {
  assert_parse_eq(docstring(inp("  /// my doc string\n")), " my doc string");

  assert_parse_eq(
    decl(inp("/// Some doc\n struct A {}")),
    adlast::Decl{
      name: adlast::Spanned::new("A".to_string(), adlast::Span::new(21, 22)),
      version: Maybe::nothing(),
      annotations:  Map::from_iter(vec![
        (docstring_scoped_name(), serde_json::Value::from(" Some doc")),
      ]),
      r#type: adlast::DeclType::Struct(adlast::Struct{
        type_params: Vec::new(),
        fields: vec![],
      }),
    },
  );

  assert_parse_eq(
    decl(inp("/// Some doc\n /// with line 2\n struct A {}")),
    adlast::Decl{
      name: adlast::Spanned::new("A".to_string(), adlast::Span::new(38, 39)),
      version:  Maybe::nothing(),
      annotations:  Map::from_iter(vec![
        (mk_scoped_name("sys.annotations", "Doc"), serde_json::Value::from(" Some doc\n with line 2")),
      ]),
      r#type: adlast::DeclType::Struct(adlast::Struct{
        type_params: Vec::new(),
        fields: vec![],
      }),
    },
  );
}

#[test]
fn parse_empty_module() {
  let pm =  raw_module(inp("module x {\n}"));
  if let Ok((_i, (m, _))) = pm  {
    assert_eq!( m.name.value, "x".to_string());
  } else {
    panic!("Failed to parse module" );
  }
}

#[test]
fn parse_json() {
  assert_parse_eq( json(inp("null")), serde_json::Value::Null);

  assert_parse_eq( json(inp("true")), serde_json::Value::Bool(true));
  assert_parse_eq( json(inp("false")), serde_json::Value::Bool(false));
  assert_parse_eq( json(inp("true")), serde_json::Value::Bool(true));

  assert_parse_eq( json(inp("45")), serde_json::Value::from(45u32));
  assert_parse_eq( json(inp("+45")), serde_json::Value::from(45u32));
  assert_parse_eq( json(inp("-45")), serde_json::Value::from(-45i32));
  assert_parse_eq( json(inp("45.2")), serde_json::Value::from(45.2f64));
  assert_parse_eq( json(inp("+45.2")), serde_json::Value::from(45.2f64));
  assert_parse_eq( json(inp("-45.2")), serde_json::Value::from(-45.2f64));

  assert_parse_eq( json(inp("\"\"")), serde_json::Value::String("".to_string()));
  assert_parse_eq( json(inp("\"xyz\"")), serde_json::Value::String("xyz".to_string()));

  assert_parse_eq( json(inp("\"\\\"\"")), serde_json::Value::String("\"".to_string()));
  assert_parse_eq( json(inp("\"\\\\\"")), serde_json::Value::String("\\".to_string()));
  assert_parse_eq( json(inp("\"\\n\"")), serde_json::Value::String("\n".to_string()));

  assert_parse_eq( json(inp("[]")), serde_json::Value::Array(Vec::new()));
  assert_parse_eq( json(inp("[ 45 ]")), serde_json::Value::Array(vec![
    serde_json::Value::from(45u32)
  ]));

  assert_parse_eq( json(inp("{}")), serde_json::Value::Object(serde_json::Map::new()));
  assert_parse_eq( json(inp(r#" {"f1": true, "f2": null}"#)), serde_json::Value::Object(mk_json_map(vec!(
    ("f1".to_owned(), serde_json::Value::Bool(true)),
    ("f2".to_owned(), serde_json::Value::Null),
  ))));
}

  #[test]
  fn parse_test_adl_files() {
    assert_module_file_ok("../../haskell/compiler/tests/test1/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test2/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test3/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test4/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test5/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test6/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test7/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test8/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test9/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test10/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test11/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test12/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test13/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test14/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test15/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test16/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test16/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test16/input/test2.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test17/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test18/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test19/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test20/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test21/input/test.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test22/input/test22a.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test22/input/test22b.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test23/input/test23.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test24/input/test24.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test25/input/admin.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test26/input/test26.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test27/input/test27.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test27/input/test27a.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test28/input/test28.adl");
    assert_module_file_ok("../../haskell/compiler/tests/test29/input/test29.adl");
  }

  fn inp (s: &str) -> Input {
    LocatedSpan::new(s)
  }
  
  fn assert_parse_eq<T>(pr: Res<Input, T>, v:T) 
    where T: std::fmt::Debug+PartialEq {
    match pr {
      Ok((i,pv)) => {
        assert_eq!(pv, v);
        assert!(i.is_empty());
      }
      Err(e) => {
        panic!("Unexpected parse failure: {}", e);

      }
    }
  }

  fn assert_parse_eq_2<T>(pr: Res<Input, T>, v:T, remaining: &str) 
    where T: std::fmt::Debug+PartialEq {
    match pr {
      Ok((i,pv)) => {
        assert_eq!(pv, v);
        assert_eq!(*i.fragment(), remaining);
      }
      Err(e) => {
        panic!("Unexpected parse failure: {}", e);

      }
    }
}

fn assert_module_file_ok(path: &str) {
  let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  d.push(path);
  let content = fs::read_to_string(d).expect(&format!("Failed to read file: {}", path) );
  let content_str: &str = &content;
  let parse_result = raw_module(inp(content_str));
  let err =  parse_result.err().and_then(|e| {
    match e {
      Err::Error(e) => Some(e),
      Err::Failure(e) => Some(e),
      Err::Incomplete(_e) => None,
    }
  });

  assert_eq!( err, Option::None);
}

fn mk_json_map(vs: Vec<(String,serde_json::Value)>) -> serde_json::Map<String, serde_json::Value> {
  let mut map = serde_json::Map::new();
  for (k,jv) in vs {
    map.insert(k,jv);
  }
  map
}
