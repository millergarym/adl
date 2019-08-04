module github.com/timbod7/adl/go

go 1.12

replace (
	github.com/dave/jennifer => /home/garym/devel/github.com/dave/jennifer
	github.com/wxio/tron-go => /home/garym/devel/wxio/tron/go
	golang.org/x/tools => github.com/wxio/tools v0.1.0
)

require (
	github.com/dave/jennifer v1.3.0
	github.com/wxio/tron-go v0.0.0-00010101000000-000000000000
)
