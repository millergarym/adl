#!/bin/bash -xe

RUST_COMPILER_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
cd $RUST_COMPILER_DIR

ADL_STDLIB_DIR=../../adl/stdlib
ADL_ADLC_DIR=../../adl/tools

adlc rust \
  --no-overwrite \
  --verbose \
  --generate-transitive \
  --outputdir ./src \
  --module adlgen \
  --runtime-module adlrt \
  --include-rt \
  --searchdir $ADL_STDLIB_DIR \
  --searchdir $ADL_ADLC_DIR \
  $ADL_STDLIB_DIR/sys/adlast2.adl \
  $ADL_ADLC_DIR/adlc/workspace.adl \
  $ADL_ADLC_DIR/adlc/bundle.adl \
  $ADL_ADLC_DIR/adlc/testing_table.adl

# ADL_DIR=../../adl

# adlc rust \
#   --no-overwrite \
#   --verbose \
#   --generate-transitive \
#   --outputdir ./src \
#   --module adlgen_dev \
#   --runtime-module adlrt \
#   --searchdir  $ADL_DIR/stdlib \
#   $ADL_DIR/adlc/adlc/testing_table.adl
