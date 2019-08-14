package main

import (
	"sort"
	"strings"

	"github.com/golangq/q"

	"github.com/dave/jennifer/jen"
	"github.com/wxio/tron-go/adl"
)

func ADL2Jen(amod *adl.Module) *jen.File {
	jf := jen.NewFile(amod.Name)
	names := []string{}
	for name, _ := range amod.Decls {
		names = append(names, name)
	}
	sort.Sort(sort.StringSlice(names))
	for _, name := range names {
		q.Q(name)
		decl := amod.Decls[name]
		lname := strings.ToLower(name)
		uname := strings.ToUpper(name)
		if decl.Type.Struct != nil {
			code := []jen.Code{}
			for _, fld := range decl.Type.Struct.Field {
				q.Q(fld)
				jf := jen.Id(strings.ToUpper(fld.Name)).Params()
				if fld.TypeExpr.TypeRef.Primitive != nil {
					// fmt.Printf("%+v\n", fld.TypeExpr.TypeRef)
					switch *fld.TypeExpr.TypeRef.Primitive {
					case "Int32":
						q.Q("0", fld.Name, *fld.TypeExpr.TypeRef.Primitive)
						code = append(code, jf.Int32())
					case "String":
						q.Q("1", fld.Name, *fld.TypeExpr.TypeRef.Primitive)
						code = append(code, jf.String())
					default:
						q.Q("2", fld.Name, *fld.TypeExpr.TypeRef.Primitive)
					}
				}
			}
			jf.Type().Id(uname).Interface(code...)

			// jen.Id("X").Params().Int32(),
			// jen.Id("Y").Params().String(),
			jf.Type().Id(lname).Struct(
				jen.Id("x").Int32(),
				jen.Id("y").String(),
			)
			jf.Var().Id("_").Id(uname).Op("=").Id(lname).Block()

			jf.Func().Params(
				jen.Id("obj").Id(lname),
			).Id("X").Params().Int32().Block(
				jen.Return(jen.Id("obj").Dot("x")),
			)

			jf.Func().Params(
				jen.Id("obj").Id(lname),
			).Id("Y").Params().String().Block(
				jen.Return(jen.Id("obj").Dot("y")),
			)

			jf.Func().Id("New"+uname).Params(
				jen.Id("x").Int32(),
				jen.Id("y").String(),
			).Id(uname).Block(
				jen.Return(jen.Id(lname).Values(jen.Dict{
					jen.Id("x"): jen.Id("x"),
					jen.Id("y"): jen.Id("y"),
				})),
			)
		}
	}
	return jf
}
