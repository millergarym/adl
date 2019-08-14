package main

// go install github.com/francoispqt/gojay/gojay
// gojay -s .  -t s1 -o output.go

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"log"

	// . "github.com/dave/jennifer/jen"
	test2 "ago/x"

	"github.com/francoispqt/gojay"
	"github.com/wxio/tron-go/adl"
)

func main() {
	astj, _ := ioutil.ReadFile("ast.json")
	amods := map[string]*adl.Module{}
	err := json.Unmarshal(astj, &amods)
	if err != nil {
		log.Fatalf("%v\n", err)
	}
	// fj := ADL2Jen(amods["test2"])
	// fmt.Printf("%#v", fj)
	TemplADL(amods["test2"])

	s1 := test2.NewS1_builder().
		X(1).
		Y("sdfa").
		S1()

	by, err := gojay.MarshalJSONObject(s1)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Println(string(by))

	s11 := test2.NewS1(0, "")
	err = gojay.Unmarshal(by, s11)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("%v\n", s11)

	// var out []byte
	// // var err error
	// if out, err = json.MarshalIndent(&s1, "", "  "); err != nil {
	// 	fmt.Printf("error : %v\n", err)
	// } else {
	// 	fmt.Printf("'%v'\n", string(out))
	// }
	// if s1_1, err := S1_UnmarshalJSON(out); err != nil {
	// 	fmt.Printf("unmarshal error : %v\n", err)
	// } else {
	// 	fmt.Printf("'%+v'\n", s1_1)
	// }

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
