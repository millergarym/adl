# Developer notes

This document is for notes when developing this module.

## Useful commands


```
cargo +nightly test --package compiler --lib -- cli::tsgen::tests::generate_ts_from_test_files --exact --nocapture 
```

```
cargo +nightly run ast --searchdir ../../adl/stdlib --searchdir ../../adl/tests/test30 test30_04
```

```
adlc ast --searchdir ../../adl/stdlib  --searchdir ../../adl/tests/test4 --combined-output=out/test4.ast.json ../../adl/tests/test30/test30_04.adl
```


```
cargo +nightly run tsgen --searchdir ../../adl/stdlib --searchdir ../../adl/tests/test3 --outdir ts-src --manifest ts-src/manifest.json --capitalize-branch-names-in-types  test3
```

```
code -d out/test4.ast.json rustout/test4.ast.json
```