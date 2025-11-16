mod generator;
mod indexer;

use crate::generator::Generator;
use futures_util::TryStreamExt;
use mongodb::{
    Client as MongoClient,
    bson::{self, doc},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env};
use tracing::info;
use url::form_urlencoded;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct FlatDocument {
    #[serde(rename = "_id")]
    id: bson::oid::ObjectId,
    item_type: i32,
    unique: Option<String>,
    name: Option<String>,
    name_japanese: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    dotenvy::dotenv()?;

    let mongo_client = MongoClient::with_uri_str(env::var("MONGO_URI")?).await?;
    let src_cl = mongo_client
        .database(&env::var("MONGO_DB")?)
        .collection::<FlatDocument>(&env::var("MONGO_CL")?);

    let src_list = src_cl
        .find(doc! {})
        .await?
        .try_collect::<Vec<FlatDocument>>()
        .await?;

    let base_url_txt = env::var("BASE_URL_TXT")?;

    let mut generator = Generator::new(1000, &env::var("LAST_MOD")?).await?;
    let mut gen_map = HashMap::new();
    for doc in src_list {
        if doc.item_type != 1 {
            continue;
        }

        if let Some(name) = doc.name {
            if !gen_map.contains_key(&name) {
                let mut q = form_urlencoded::Serializer::new(String::new());
                q.append_pair("keyword", &name)
                    .append_pair("item-type", "all");
                let url = format!("{base_url_txt}?{}", q.finish());

                generator.write(url).await?;
                gen_map.insert(name, true);
            }
        };

        if let Some(name_japanese) = doc.name_japanese {
            if !gen_map.contains_key(&name_japanese) {
                let mut q = form_urlencoded::Serializer::new(String::new());
                q.append_pair("keyword", &name_japanese)
                    .append_pair("item-type", "all")
                    .append_pair("lang", "ja");
                let url = format!("{base_url_txt}?{}", q.finish());

                generator.write(url).await?;
                gen_map.insert(name_japanese, true);
            }
        };
    }

    generator.finish().await?;
    generator.flush().await?;

    info!("done");
    Ok(())
}
