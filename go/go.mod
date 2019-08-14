module github.com/timbod7/adl/go

go 1.12

replace (
	github.com/dave/jennifer => /home/garym/devel/github.com/dave/jennifer
	github.com/wxio/tron-go => /home/garym/devel/wxio/tron/go
	golang.org/x/tools => github.com/wxio/tools v0.1.0
)

require (
	github.com/dave/jennifer v1.3.0
	github.com/francoispqt/gojay v1.2.13 // indirect
	github.com/golang/glog v0.0.0-20160126235308-23def4e6c14b
	github.com/golangq/q v1.0.7
	github.com/wxio/tron-go v0.0.0-00010101000000-000000000000
)
