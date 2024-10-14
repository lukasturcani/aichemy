# Show all available recipes
default:
  @just --list

# Populate a MongoDB database with test data
init_nomad_nmr_test_db nomad-datastore mongo-uri="mongodb://localhost:27017":
  cargo run --bin init_nomad_nmr_test_db -- {{nomad-datastore}} {{mongo-uri}}

# Run all code checks
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

  echo
  (set -x; RUSTDOCFLAGS="-D warnings" cargo doc --no-deps)
  test $? = 0

  test $error = 0
