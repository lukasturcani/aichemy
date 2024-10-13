use std::collections::HashMap;

use clap::Parser;
use mongodb::{
    bson::{doc, Bson},
    Client, Database,
};

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
    let parameter_sets = add_parameter_sets(&db, &instruments).await?;
    let groups = add_groups(&db).await?;
    let users = add_users(&db, &groups).await?;
    let experiments = add_experiments(&db, &instruments, &groups, &users).await?;
    Ok(())
}

async fn add_instruments(db: &Database) -> Result<HashMap<usize, Bson>, anyhow::Error> {
    let collection = db.collection("instruments");
    collection.delete_many(doc! {}).await?;
    Ok(collection
        .insert_many([
            doc! {
                "status": {
                    "statusTable": [],
                },
                "name": "instrument-1",
                "isActive": true,
                "available": true,
                "capacity": 60,
                "dayAllowance": 20,
                "nightAllowance": 195,
                "overheadTime": 255,
                "cost": 3
            },
            doc! {
                "status": {
                    "statusTable": [],
                },
                "name": "instrument-2",
                "isActive": false,
                "available": false,
                "capacity": 60,
                "cost": 2,
            },
            doc! {
                "name": "instrument-3",
                "isActive": true,
                "available": true,
                "capacity": 24,
                "status": {
                    "statusTable": [],
                },
                "cost": 2,
                "dayAllowance": 20,
                "nightAllowance": 195,
                "overheadTime": 255,
                "nightEnd": "09:00",
                "nightStart": "19:00",
            },
        ])
        .await?
        .inserted_ids)
}

async fn add_parameter_sets(
    db: &Database,
    instruments: &HashMap<usize, Bson>,
) -> Result<HashMap<usize, Bson>, anyhow::Error> {
    let collection = db.collection("parameter_sets");
    collection.delete_many(doc! {}).await?;
    Ok(collection
        .insert_many([
            doc! {
                "name": "parameter-set-1",
                "availableOn": [instruments[&0].clone()],
            },
            doc! {
                "name": "parameter-set-2",
                "availableOn": [instruments[&1].clone()],
            },
            doc! {
                "name": "parameter-set-3",
                "availableOn": [instruments[&0].clone(), instruments[&2].clone()],
            },
        ])
        .await?
        .inserted_ids)
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
    instruments: &HashMap<usize, Bson>,
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
                    "id": instruments[&0].clone(),
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
                    "id": instruments[&0].clone(),
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
        ])
        .await?
        .inserted_ids)
}
