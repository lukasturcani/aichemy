default:
  @just --list

init_nomad_nmr_test_db nomad-datastore mongo-uri="mongodb://localhost:27017":
  cargo run --bin init_nomad_nmr_test_db -- {{nomad-datastore}} {{mongo-uri}}
