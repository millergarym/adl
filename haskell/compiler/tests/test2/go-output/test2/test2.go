package main

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"strconv"
	"strings"
)

type S0 struct{}

type S1 interface {
	X() int32
	Y() string
}
type s1 struct {
	x int32
	y string
}

var _ S1 = s1{}

func (obj s1) X() int32  { return obj.x }
func (obj s1) Y() string { return obj.y }

func NewS1(x int32, y string) S1 {
	return s1{x: x, y: y}
}

type S1_builder_X interface {
	X(int32) S1_builder_Y
}
type S1_builder_Y interface {
	Y(string) S1_builder
}
type S1_builder interface {
	S1() S1
}
type s1_builder struct {
	obj s1
}

func (br *s1_builder) X(x int32) S1_builder_Y {
	br.obj.x = x
	return br
}
func (br *s1_builder) Y(y string) S1_builder {
	br.obj.y = y
	return br
}
func (br s1_builder) S1() S1 {
	return br.obj
}
func NewS1_builder() S1_builder_X {
	return &s1_builder{obj: s1{}}
}

func (obj s1) MarshalJSON() ([]byte, error) {
	buf := bytes.Buffer{}
	buf.WriteString("{ ")
	buf.WriteString(`"`)
	buf.WriteString(`x`)
	buf.WriteString(`"`)
	buf.WriteString(":")
	buf.WriteString(strconv.FormatInt(int64(obj.x), 10))
	buf.WriteString(`,`)
	buf.WriteString(`"`)
	buf.WriteString(`y`)
	buf.WriteString(`"`)
	buf.WriteString(`:`)
	buf.WriteString(`"`)
	buf.WriteString(obj.y)
	buf.WriteString(`"`)
	buf.WriteString("}")
	return buf.Bytes(), nil
}
func (obj *s1) UnmarshalJSON(buf []byte) error {
	var dyn interface{}
	if err := json.Unmarshal(buf, &dyn); err != nil {
		return err
	}
	if mp, ok := dyn.(map[string]interface{}); !ok {
		return fmt.Errorf("can't umarshal %T", dyn)
	} else {
		errs := []string{}
		if val, ex := mp["x"]; !ex {
			errs = append(errs, "missing field 'x'")
		} else {
			if fval, ok := val.(float64); !ok {
				errs = append(errs, fmt.Sprintf("type error for field 'x'. expecting int32 received %T", val))
			} else {
				obj.x = int32(fval)
			}
		}
		delete(mp, "x")
		//
		if val, ex := mp["y"]; !ex {
			errs = append(errs, "missing field 'y'")
		} else {
			if obj.y, ok = val.(string); !ok {
				errs = append(errs, fmt.Sprintf("type error for field 'y'. expecting string received %T", val))
			}
		}
		delete(mp, "y")
		if len(mp) != 0 {
			errs = append(errs, fmt.Sprintf("remaining fields '%v'", mp))
		}
		if len(errs) != 0 {
			return errors.New(strings.Join(errs, ", "))
		}
	}
	return nil
}

func S1_UnmarshalJSON(buf []byte) (S1, error) {
	s1 := s1{}
	err := json.Unmarshal(buf, &s1)
	return s1, err
}

func main() {
	s1 := NewS1_builder().
		X(1).
		Y("sdfa").
		S1()
	var out []byte
	var err error
	if out, err = json.MarshalIndent(&s1, "", "  "); err != nil {
		fmt.Printf("error : %v\n", err)
	} else {
		fmt.Printf("'%v'\n", string(out))
	}
	if s1_1, err := S1_UnmarshalJSON(out); err != nil {
		fmt.Printf("unmarshal error : %v\n", err)
	} else {
		fmt.Printf("'%+v'\n", s1_1)
	}
}
