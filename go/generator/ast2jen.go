package main

import (
	"strings"

	"github.com/dave/jennifer/jen"
	"github.com/wxio/tron-go/adl"
)

func ADL2Jen(amod *adl.Module) *jen.File {
	jf := jen.NewFile(amod.Name)
	for name, decl := range amod.Decls {
		lname := strings.ToLower(name)
		uname := strings.ToUpper(name)
		if decl.Type.Struct != nil {
			jf.Type().Id(lname).Struct(
				jen.Id("x").Int32(),
				jen.Id("y").String(),
			)
			jf.Type().Id(uname).Interface(
				jen.Id("X").Params().Int32(),
				jen.Id("Y").Params().String(),
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
