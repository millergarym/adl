package main

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"log"

	// . "github.com/dave/jennifer/jen"
	"github.com/wxio/tron-go/adl"
)

func main() {
	astj, _ := ioutil.ReadFile("ast.json")
	amods := map[string]*adl.Module{}
	err := json.Unmarshal(astj, &amods)
	if err != nil {
		log.Fatalf("%v\n", err)
	}
	fj := ADL2Jen(amods["test2"])
	fmt.Printf("%#v", fj)

	// f := NewFile("main")
	// f.Type().Id("s1").Struct(
	// 	Id("x").Int32(),
	// 	Id("y").String(),
	// )
	// f.Type().Id("S1").Interface(
	// 	Id("X").Params().Int32(),
	// 	Id("Y").Params().String(),
	// )
	// f.Var().Id("_").Id("S1").Op("=").Id("s1").Block()

	// f.Func().Params(
	// 	Id("obj").Id("s1"),
	// ).Id("X").Params().Int32().Block(
	// 	Return(Id("obj").Dot("x")),
	// )

	// f.Func().Params(
	// 	Id("obj").Id("s1"),
	// ).Id("Y").Params().String().Block(
	// 	Return(Id("obj").Dot("y")),
	// )

	// f.Func().Id("NewS1").Params(
	// 	Id("x").Int32(),
	// 	Id("y").String(),
	// ).Id("S1").Block(
	// 	Return(Id("s1").Values(Dict{
	// 		Id("x"): Id("x"),
	// 		Id("y"): Id("y"),
	// 	})),
	// )

	// f.Func().Id("main").Params().Block(
	// 	Qual("fmt", "Println").Call(Lit("Hello, world")),
	// )
	// fmt.Printf("%#v", f)
}
