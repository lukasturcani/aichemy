default:
  @just --list

init_nomad_nmr_test_db nomad-datastore mongo-uri="mongodb://localhost:27017":
  cargo run --bin init_nomad_nmr_test_db -- {{nomad-datastore}} {{mongo-uri}}

check:
  #!/usr/bin/env bash

  error=0
  trap error=1 ERR

  echo
  (set -x; cargo fmt --check)
  test $? = 0

  echo
  (set -x; cargo check --all-features --all-targets)
  test $? = 0

  echo
  (set -x; cargo clippy --all-features --all-targets -- -D warnings)
  test $? = 0

  echo
  (set -x; cargo test --all-features --all-targets)
  test $? = 0

  echo
  (set -x; cargo test --doc)
  test $? = 0


  #TODO: check if building docs generated separates warnings and if you can turn them to errors
  #TODO: Fix existing doc building warnigns


  test $error = 0
