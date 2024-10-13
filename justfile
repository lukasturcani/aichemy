default:
  @just --list

clear_nomad_nmr_test_db mongo-container:
  docker exec {{mongo-container}} mongosh --eval "db.getSiblingDB('nomad').instruments.deleteMany({})"
  docker exec {{mongo-container}} mongosh --eval "db.getSiblingDB('nomad').parametersets.deleteMany({})"
  docker exec {{mongo-container}} mongosh --eval "db.getSiblingDB('nomad').groups.deleteMany({groupName: {\$ne: 'default'}})"

init_nomad_nmr_test_db mongo-container backend-url="http://localhost:8080": (clear_nomad_nmr_test_db mongo-container)
  cargo run --bin init_nomad_nmr_test_db -- {{backend-url}}
