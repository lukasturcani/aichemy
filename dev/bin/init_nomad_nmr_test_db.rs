use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use clap::Parser;
use mongodb::{
    bson::{doc, oid::ObjectId, DateTime},
    Client, Database,
};
use serde::{Deserialize, Serialize};
use zip::{write::SimpleFileOptions, ZipWriter};

#[derive(Parser)]
struct Cli {
    /// Path to the NOMAD datastore
    datastore: PathBuf,

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
    add_experiments(&db, &instruments, &groups, &users, &cli.datastore).await?;
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
    #[serde(rename = "groupName")]
    name: String,
    #[serde(rename = "isActive")]
    is_active: bool,
    description: String,
    #[serde(rename = "isBatch")]
    is_batch: bool,
    #[serde(rename = "dataAccess")]
    data_access: String,
}

async fn add_groups(db: &Database) -> Result<HashMap<usize, GroupId>, anyhow::Error> {
    let collection = db.collection("groups");
    collection
        .delete_many(doc! {"groupName": {"$ne": "default"}})
        .await?;
    let ids = collection
        .insert_many([
            Group {
                id: None,
                name: "group-1".into(),
                is_active: true,
                description: "Test group 1".into(),
                is_batch: false,
                data_access: "user".into(),
            },
            Group {
                id: None,
                name: "test-admins".into(),
                is_active: true,
                description: "Admins test group".into(),
                is_batch: true,
                data_access: "user".into(),
            },
        ])
        .await?
        .inserted_ids;
    Ok(ids
        .into_iter()
        .map(|id| (id.0, GroupId(id.1.as_object_id().unwrap())))
        .collect())
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct UserId(ObjectId);

#[derive(Serialize, Deserialize, Debug, Clone)]
struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    username: String,
    #[serde(rename = "fullName")]
    full_name: String,
    email: String,
    password: String,
    #[serde(rename = "isActive")]
    is_active: bool,
    group: GroupId,
    #[serde(rename = "accessLevel")]
    access_level: String,
}

async fn add_users(
    db: &Database,
    groups: &HashMap<usize, GroupId>,
) -> Result<HashMap<usize, UserId>, anyhow::Error> {
    let collection = db.collection("users");
    collection
        .delete_many(doc! {"username": {"$ne": "admin"}})
        .await?;
    let ids = collection
        .insert_many([
            User {
                id: None,
                username: "test1".into(),
                full_name: "Test User One".into(),
                email: "test1@test.com".into(),
                password: "t1p1".into(),
                is_active: false,
                group: groups[&0],
                access_level: "user".into(),
            },
            User {
                id: None,
                username: "test2".into(),
                full_name: "Test User Two".into(),
                email: "test2@test.com".into(),
                password: "t2p2".into(),
                is_active: false,
                group: groups[&0],
                access_level: "user".into(),
            },
            User {
                id: None,
                username: "test3".into(),
                full_name: "Test User Three".into(),
                email: "test3@test.com".into(),
                password: "t3p3".into(),
                is_active: false,
                group: groups[&1],
                access_level: "admin".into(),
            },
        ])
        .await?
        .inserted_ids;
    Ok(ids
        .into_iter()
        .map(|id| (id.0, UserId(id.1.as_object_id().unwrap())))
        .collect())
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct ExperimentId(ObjectId);

#[derive(Serialize, Deserialize, Debug, Clone)]
struct InstrumentInfo {
    id: InstrumentId,
    name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UserInfo {
    id: UserId,
    username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GroupInfo {
    id: GroupId,
    name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Experiment {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    #[serde(rename = "expId")]
    exp_id: String,
    instrument: InstrumentInfo,
    user: UserInfo,
    group: GroupInfo,
    #[serde(rename = "datasetName")]
    dataset_name: String,
    status: String,
    title: String,
    #[serde(rename = "parameterSet")]
    parameter_set: ParameterSetId,
    #[serde(rename = "expNo")]
    exp_no: String,
    holder: String,
    #[serde(rename = "dataPath")]
    data_path: PathBuf,
    solvent: String,
    #[serde(rename = "submittedAt", skip_serializing_if = "Option::is_none")]
    submitted_at: Option<DateTime>,
}

async fn add_experiments(
    db: &Database,
    instruments: &HashMap<usize, InstrumentId>,
    groups: &HashMap<usize, GroupId>,
    users: &HashMap<usize, UserId>,
    datastore: &Path,
) -> Result<HashMap<usize, ExperimentId>, anyhow::Error> {
    let collection = db.collection::<Experiment>("experiments");
    collection.delete_many(doc! {}).await?;
    let experiments = [
        Experiment {
            id: None,
            exp_id: "2106231050-2-1-test1-10".into(),
            instrument: InstrumentInfo {
                id: instruments[&0],
                name: "instrument-1".into(),
            },
            user: UserInfo {
                id: users[&0],
                username: "test1".into(),
            },
            group: GroupInfo {
                id: groups[&0],
                name: "group-1".into(),
            },
            dataset_name: "2106231050-2-1-test1".into(),
            status: "Archived".into(),
            title: "Test Exp 1".into(),
            parameter_set: ParameterSetId(ObjectId::new()),
            exp_no: "10".into(),
            holder: "2".into(),
            data_path: PathBuf::from("."),
            solvent: "CDCl3".into(),
            submitted_at: None,
        },
        Experiment {
            id: None,
            exp_id: "2106231050-2-1-test1-11".into(),
            instrument: InstrumentInfo {
                id: instruments[&0],
                name: "instrument-1".into(),
            },
            user: UserInfo {
                id: users[&0],
                username: "test1".into(),
            },
            group: GroupInfo {
                id: groups[&0],
                name: "group-1".into(),
            },
            dataset_name: "2106231050-2-1-test1".into(),
            status: "Archived".into(),
            title: "Test Exp 1".into(),
            parameter_set: ParameterSetId(ObjectId::new()),
            exp_no: "11".into(),
            holder: "2".into(),
            data_path: PathBuf::from("."),
            solvent: "CDCl3".into(),
            submitted_at: None,
        },
        Experiment {
            id: None,
            exp_id: "2106231055-3-2-test2-10".into(),
            instrument: InstrumentInfo {
                id: instruments[&1],
                name: "instrument-2".into(),
            },
            user: UserInfo {
                id: users[&1],
                username: "test2".into(),
            },
            group: GroupInfo {
                id: groups[&0],
                name: "group-1".into(),
            },
            dataset_name: "2106231055-3-2-test2".into(),
            status: "Archived".into(),
            title: "Test Exp 3".into(),
            parameter_set: ParameterSetId(ObjectId::new()),
            exp_no: "10".into(),
            holder: "3".into(),
            data_path: PathBuf::from("."),
            solvent: "C6D6".into(),
            submitted_at: None,
        },
        Experiment {
            id: None,
            exp_id: "2106231100-10-2-test3-10".into(),
            instrument: InstrumentInfo {
                id: instruments[&2],
                name: "instrument-2".into(),
            },
            user: UserInfo {
                id: users[&2],
                username: "test3".into(),
            },
            group: GroupInfo {
                id: groups[&0],
                name: "group-1".into(),
            },
            dataset_name: "2106231100-10-2-test3".into(),
            status: "Archived".into(),
            title: "Test Exp 4".into(),
            parameter_set: ParameterSetId(ObjectId::new()),
            exp_no: "10".into(),
            holder: "10".into(),
            data_path: PathBuf::from("."),
            solvent: "C6D6".into(),
            submitted_at: None,
        },
        Experiment {
            id: None,
            exp_id: "2106240012-10-2-test2-10".into(),
            instrument: InstrumentInfo {
                id: instruments[&2],
                name: "instrument-3".into(),
            },
            user: UserInfo {
                id: users[&2],
                username: "test3".into(),
            },
            group: GroupInfo {
                id: groups[&0],
                name: "group-1".into(),
            },
            dataset_name: "2106240012-10-2-test2".into(),
            status: "Available".into(),
            title: "Test Exp 5".into(),
            parameter_set: ParameterSetId(ObjectId::new()),
            exp_no: "10".into(),
            holder: "10".into(),
            data_path: PathBuf::from("."),
            solvent: "C6D6".into(),
            submitted_at: None,
        },
        Experiment {
            id: None,
            exp_id: "2106241100-10-2-test3-10".into(),
            instrument: InstrumentInfo {
                id: instruments[&2],
                name: "instrument-3".into(),
            },
            user: UserInfo {
                id: users[&2],
                username: "test3".into(),
            },
            group: GroupInfo {
                id: groups[&1],
                name: "group-2".into(),
            },
            dataset_name: "2106241100-10-2-test3".into(),
            status: "Archived".into(),
            title: "Test Exp 6".into(),
            parameter_set: ParameterSetId(ObjectId::new()),
            exp_no: "10".into(),
            holder: "10".into(),
            data_path: PathBuf::from("."),
            solvent: "CDCl3".into(),
            submitted_at: None,
        },
        Experiment {
            id: None,
            exp_id: "2106241100-10-2-test4-1".into(),
            instrument: InstrumentInfo {
                id: instruments[&2],
                name: "instrument-3".into(),
            },
            user: UserInfo {
                id: users[&2],
                username: "test3".into(),
            },
            group: GroupInfo {
                id: groups[&1],
                name: "group-2".into(),
            },
            dataset_name: "2106241100-10-2-test4".into(),
            status: "Archived".into(),
            title: "Test Exp 7".into(),
            parameter_set: ParameterSetId(ObjectId::new()),
            exp_no: "1".into(),
            holder: "11".into(),
            data_path: PathBuf::from("."),
            solvent: "CDCl3".into(),
            submitted_at: Some(DateTime::parse_rfc3339_str("2024-01-01T00:00:00.000Z")?),
        },
    ];
    let ids = collection.insert_many(&experiments).await?.inserted_ids;
    for experiment in experiments {
        let file = File::create(datastore.join(format!("{}.zip", experiment.exp_id)))?;
        let mut zip = ZipWriter::new(file);
        zip.start_file(
            format!("{}.json", experiment.exp_id),
            SimpleFileOptions::default(),
        )?;
        zip.write_all(serde_json::to_string(&experiment.exp_id)?.as_bytes())?;
        zip.finish()?;
    }
    Ok(ids
        .into_iter()
        .map(|id| (id.0, ExperimentId(id.1.as_object_id().unwrap())))
        .collect())
}
