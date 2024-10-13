use std::collections::HashMap;

use clap::Parser;
use mongodb::{
    bson::{doc, oid::ObjectId, Bson},
    Client, Database,
};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
struct Cli {
    /// The URI to the MongoDB server
    uri: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();
    let client = Client::with_uri_str(&cli.uri).await?;
    let db = client.database("nomad");
    let instruments = add_instruments(&db).await?;
    add_parameter_sets(&db, &instruments).await?;
    let groups = add_groups(&db).await?;
    let users = add_users(&db, &groups).await?;
    add_experiments(&db, &instruments, &groups, &users).await?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct InstrumentId(ObjectId);

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Instrument {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    name: String,
    #[serde(rename = "isActive")]
    is_active: bool,
    available: bool,
    capacity: i64,
    #[serde(rename = "dayAllowance")]
    day_allowance: i64,
    #[serde(rename = "nightAllowance")]
    night_allowance: i64,
    #[serde(rename = "overheadTime")]
    overhead_time: i64,
    cost: i64,
}

async fn add_instruments(db: &Database) -> Result<HashMap<usize, InstrumentId>, anyhow::Error> {
    let collection = db.collection("instruments");
    collection.delete_many(doc! {}).await?;
    let ids = collection
        .insert_many([
            Instrument {
                id: None,
                name: "instrument-1".into(),
                is_active: true,
                available: true,
                capacity: 60,
                cost: 3,
                day_allowance: 20,
                night_allowance: 195,
                overhead_time: 255,
            },
            Instrument {
                id: None,
                name: "instrument-2".into(),
                is_active: false,
                available: false,
                capacity: 60,
                cost: 2,
                day_allowance: 20,
                night_allowance: 195,
                overhead_time: 255,
            },
            Instrument {
                id: None,
                name: "instrument-3".into(),
                is_active: true,
                available: true,
                capacity: 24,
                cost: 2,
                day_allowance: 20,
                night_allowance: 195,
                overhead_time: 255,
            },
        ])
        .await?
        .inserted_ids;
    Ok(ids
        .into_iter()
        .map(|id| (id.0, InstrumentId(id.1.as_object_id().unwrap())))
        .collect())
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct ParameterSetId(ObjectId);

#[derive(Serialize, Deserialize)]
struct ParameterSet {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    name: String,
    #[serde(rename = "availableOn")]
    available_on: Vec<InstrumentId>,
}

async fn add_parameter_sets(
    db: &Database,
    instruments: &HashMap<usize, InstrumentId>,
) -> Result<HashMap<usize, ParameterSetId>, anyhow::Error> {
    let collection = db.collection("parameter_sets");
    collection.delete_many(doc! {}).await?;
    let ids = collection
        .insert_many([
            ParameterSet {
                id: None,
                name: "parameter-set-1".into(),
                available_on: vec![instruments[&0]],
            },
            ParameterSet {
                id: None,
                name: "parameter-set-2".into(),
                available_on: vec![instruments[&1]],
            },
            ParameterSet {
                id: None,
                name: "parameter-set-3".into(),
                available_on: vec![instruments[&0], instruments[&2]],
            },
        ])
        .await?
        .inserted_ids;
    Ok(ids
        .into_iter()
        .map(|id| (id.0, ParameterSetId(id.1.as_object_id().unwrap())))
        .collect())
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct GroupId(ObjectId);

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Group {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    name: String,
    #[serde(rename = "isActive")]
    is_active: bool,
    description: String,
    #[serde(rename = "isBatch")]
    is_batch: bool,
    data_access: String,
}

async fn add_groups(db: &Database) -> Result<HashMap<usize, Bson>, anyhow::Error> {
    let collection = db.collection("groups");
    collection
        .delete_many(doc! {"groupName": {"$ne": "default"}})
        .await?;
    Ok(collection
        .insert_many([
            doc! {
                "groupName": "group-1",
                "isActive": true,
                "description": "Test group 1",
                "isBatch": false,
                "dataAccess": "user",
            },
            doc! {
                "groupName": "test-admins",
                "isActive": true,
                "description": "Admins test group",
                "isBatch": true,
                "dataAccess": "user",
                "exUsers": [],
            },
        ])
        .await?
        .inserted_ids)
}

async fn add_users(
    db: &Database,
    groups: &HashMap<usize, Bson>,
) -> Result<HashMap<usize, Bson>, anyhow::Error> {
    let collection = db.collection("users");
    collection
        .delete_many(doc! {"username": {"$ne": "admin"}})
        .await?;
    Ok(collection
        .insert_many([
            doc! {
                "username": "test1",
                "fullName": "Test User One",
                "email": "test1@test.com",
                "password": "t1p1",
                "isActive": false,
                "group": groups[&0].clone(),
                "accessLevel": "user",
                "tokens": [],
            },
            doc! {
                "username": "test2",
                "fullName": "Test User Two",
                "email": "test2@test.com",
                "password": "t2p2",
                "isActive": false,
                "group": groups[&0].clone(),
                "accessLevel": "user",
                "tokens": [],
            },
            doc! {
                "username": "test3",
                "fullName": "Test User Three",
                "email": "test3@test.com",
                "password": "t3p3",
                "isActive": false,
                "group": groups[&1].clone(),
                "accessLevel": "admin",
                "tokens": [],
            },
        ])
        .await?
        .inserted_ids)
}

async fn add_experiments(
    db: &Database,
    instruments: &HashMap<usize, InstrumentId>,
    groups: &HashMap<usize, Bson>,
    users: &HashMap<usize, Bson>,
) -> Result<HashMap<usize, Bson>, anyhow::Error> {
    let collection = db.collection("experiments");
    collection.delete_many(doc! {}).await?;
    Ok(collection
        .insert_many([
            doc! {
                "expId": "2106231050-2-1-test1-10",
                "instrument": {
                    "name": "instrument-1",
                    "id": instruments[&0].0.clone(),
                },
                "user": {
                    "username": "test1",
                    "id": users[&0].clone(),
                },
                "group": {
                    "name": "group-1",
                    "id": groups[&0].clone(),
                },
                "datasetName": "2106231050-2-1-test1",
                "status": "Archived",
                "title": "Test Exp 1",
                "parameterSet": "parameter-set-1",
                "expNo": "10",
                "holder": "2",
                "dataPath": "./test/path",
                "solvent": "CDCl3",
            },
            doc! {
                "expId": "2106231050-2-1-test1-11",
                "instrument": {
                    "name": "instrument-1",
                    "id": instruments[&0].0.clone(),
                },
                "user": {
                    "username": "test1",
                    "id": users[&0].clone(),
                },
                "group": {
                    "name": "group-1",
                    "id": groups[&0].clone(),
                },
                "datasetName": "2106231050-2-1-test1",
                "status": "Archived",
                "title": "Test Exp 1",
                "parameterSet": "parameter-set-2",
                "expNo": "11",
                "holder": "2",
                "dataPath": "./test/path",
                "solvent": "CDCl3",
            },
            doc! {
                "expId": "2106231055-3-2-test2-10",
                "instrument": {
                    "name": "instrument-2",
                    "id": instruments[&1].0.clone(),
                },
                "user": {
                    "username": "test2",
                    "id": users[&1].clone(),
                },
                "group": {
                    "name": "group-1",
                    "id": groups[&0].clone(),
                },
                "datasetName": "2106231055-3-2-test2",
                "status": "Archived",
                "title": "Test Exp 3",
                "parameterSet": "parameter-set-2",
                "expNo": "10",
                "holder": "3",
                "dataPath": "./test/path",
                "solvent": "C6D6",
            },
            doc! {
                "expId": "2106231100-10-2-test3-10",
                "instrument": {
                    "name": "instrument-2",
                    "id": instruments[&1].0.clone(),
                },
                "user": {
                    "username": "test3",
                    "id": users[&2].clone(),
                },
                "group": {
                    "name": "group-1",
                    "id": groups[&0].clone(),
                },
                "datasetName": "2106231100-10-2-test3",
                "status": "Archived",
                "title": "Test Exp 4",
                "parameterSet": "parameter-set-1",
                "expNo": "10",
                "holder": "10",
                "dataPath": "./test/path",
                "solvent": "C6D6",
            },
            doc! {
                "expId": "2106240012-10-2-test2-10",
                "instrument": {
                    "name": "instrument-3",
                    "id": instruments[&2].0.clone(),
                },
                "user": {
                    "username": "test3",
                    "id": users[&2].clone(),
                },
                "datasetName": "2106240012-10-2-test2",
                "status": "Available",
                "title": "Test Exp 5",
                "parameterSet": "parameter-set-1",
                "expNo": "10",
                "holder": "10",
                "dataPath": "./test/path",
                "solvent": "C6D6",
            },
            doc! {
                "expId": "2106241100-10-2-test3-10",
                "instrument": {
                    "name": "instrument-3",
                    "id": instruments[&2].0.clone(),
                },
                "user": {
                    "username": "test3",
                    "id": users[&2].clone(),
                },
                "group": {
                    "name": "group-2",
                    "id": groups[&1].clone(),
                },
                "datasetName": "2106241100-10-2-test3",
                "status": "Archived",
                "title": "Test Exp 6",
                "parameterSet": "parameter-set-1",
                "expNo": "10",
                "holder": "10",
                "dataPath": "./test/path",
                "solvent": "CDCl3",
            },
            doc! {
                "expId": "2106241100-10-2-test4-1",
                "instrument": {
                    "name": "instrument-3",
                    "id": instruments[&2].0.clone(),
                },
                "user": {
                    "username": "test3",
                    "id": users[&2].clone(),
                },
                "group": {
                    "name": "group-2",
                    "id": groups[&1].clone(),
                },
                "datasetName": "2106241100-10-2-test4",
                "status": "Archived",
                "title": "Test Exp 7",
                "parameterSet": "parameter-set-1",
                "expNo": "1",
                "holder": "11",
                "dataPath": "./test/path",
                "solvent": "CDCl3",
            },
        ])
        .await?
        .inserted_ids)
}
