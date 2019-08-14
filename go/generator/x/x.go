//// Code generated by go2adl. DO NOT EDIT.
// source:

package test2

import "github.com/francoispqt/gojay"

type S1 interface {
	X() int32
	Y() string
	IsNil() bool
	UnmarshalJSONObject(dec *gojay.Decoder, k string) error
	MarshalJSONObject(enc *gojay.Encoder)
}

type s1 struct {
	x int32
	y string
}

var _ S1 = &s1{}

func (obj s1) X() int32  { return obj.x }
func (obj s1) Y() string { return obj.y }

func NewS1(x int32, y string) S1 {
	return &s1{
		x: x,
		y: y,
	}
}

type S1_builder_X interface {
	X(int32) S1_builder_Y
}

func (br *s1_builder) X(x int32) S1_builder_Y {
	br.obj.x = x
	return br
}

type S1_builder_Y interface {
	Y(string) S1_builder
}

func (br *s1_builder) Y(y string) S1_builder {
	br.obj.y = y
	return br
}

type S1_builder interface {
	S1() S1
}

type s1_builder struct {
	obj s1
}

func (br s1_builder) S1() S1 {
	return &br.obj
}
func NewS1_builder() S1_builder_X {
	return &s1_builder{obj: s1{}}
}
