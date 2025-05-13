#!/bin/bash
RUST_LOG=a2a=info cargo run --bin=a2a -- coder \
  -M a2a/test_scripts/case01/models2.txt \
  --clean a2a/test_scripts/case01/case01.clean.js \
  -r -c a2a/test_scripts/conf a2a/test_scripts/case01/case01.md >> run.csv