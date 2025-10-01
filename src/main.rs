mod generator;

use crate::generator::Generator;
// use anyhow::bail;
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

const F: &'static str = "
    <url>
        <loc>%LOC%</loc>
        <lastmod>%LASTMOD%</lastmod>
    </url>";

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

    let f = F.replace("%LASTMOD%", &env::var("LAST_MOD")?);
    // let base_url_vec = env::var("BASE_URL_VEC")?;
    let base_url_txt = env::var("BASE_URL_TXT")?;

    let mut generator = Generator::new(1000).await?;
    let mut gen_map = HashMap::new();
    for doc in src_list {
        if doc.item_type == 0 || doc.item_type >= 3 {
            continue;
        }

        // let item_type = match doc.item_type {
        //     1 => "anime",
        //     2 => "character",
        //     _ => bail!("invalid item type {}", doc.item_type),
        // };

        // if !gen_map.contains_key(&doc.id.to_hex()) {
        //     let mut q = form_urlencoded::Serializer::new(String::new());
        //     q.append_pair("item-type", item_type);
        //     let url = format!("{base_url_vec}/{}?{}", doc.id, q.finish());
        //     let url_jp = format!("{url}&lang=ja");
        //     let xml = f.replace("%LOC%", &url.replace('&', "&amp;"));
        //     let xml_jp = f.replace("%LOC%", &url_jp.replace('&', "&amp;"));

        //     generator.write(xml).await?;
        //     generator.write(xml_jp).await?;
        //     gen_map.insert(doc.id.to_hex(), true);
        // }

        if doc.item_type == 2 {
            continue;
        }

        if let Some(name) = doc.name {
            if !gen_map.contains_key(&name) {
                let mut q = form_urlencoded::Serializer::new(String::new());
                q.append_pair("keyword", &name)
                    .append_pair("item-type", "all");
                let url = format!("{base_url_txt}?{}", q.finish());
                let xml = f.replace("%LOC%", &url.replace('&', "&amp;"));

                generator.write(xml).await?;
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
                let xml = f.replace("%LOC%", &url.replace('&', "&amp;"));

                generator.write(xml).await?;
                gen_map.insert(name_japanese, true);
            }
        };
    }

    generator.flush().await?;

    info!("done");
    Ok(())
}
