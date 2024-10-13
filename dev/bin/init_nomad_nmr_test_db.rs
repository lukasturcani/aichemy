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
