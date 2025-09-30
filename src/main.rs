use anyhow::bail;
use futures_util::TryStreamExt;
use mongodb::{
    Client as MongoClient,
    bson::{self, doc},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env};
use tokio::{fs, io::AsyncWriteExt};
use tracing::info;
use url::form_urlencoded;

const HEAD: &'static str = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#;
const FOOT: &'static str = "
</urlset>
";
const F: &'static str = "
    <url>
        <loc>%LOC%</loc>
        <lastmod>%LASTMOD%</lastmod>
    </url>";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FlatDocument {
    #[serde(rename = "_id")]
    pub id: bson::oid::ObjectId,
    pub item_type: i32,
    pub unique: Option<String>,
    pub name: Option<String>,
    pub name_japanese: Option<String>,
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

    let f = F.replace("%LASTMOD%", &env::var("LAST_MOD")?);
    let base_url_vec = env::var("BASE_URL_VEC")?;
    let base_url_txt = env::var("BASE_URL_TXT")?;

    let mut sitemap_vec = fs::File::options()
        .create(true)
        .append(true)
        .open("vectorsearch.xml")
        .await?;
    let mut sitemap_vec_jp = fs::File::options()
        .create(true)
        .append(true)
        .open("vectorsearch_jp.xml")
        .await?;
    let mut sitemap_txt = fs::File::options()
        .create(true)
        .append(true)
        .open("textsearch.xml")
        .await?;
    let mut sitemap_txt_jp = fs::File::options()
        .create(true)
        .append(true)
        .open("textsearch_jp.xml")
        .await?;
    let mut sitemap_vec_2 = fs::File::options()
        .create(true)
        .append(true)
        .open("vectorsearch_2.xml")
        .await?;
    let mut sitemap_vec_jp_2 = fs::File::options()
        .create(true)
        .append(true)
        .open("vectorsearch_jp_2.xml")
        .await?;
    let mut sitemap_txt_2 = fs::File::options()
        .create(true)
        .append(true)
        .open("textsearch_2.xml")
        .await?;
    let mut sitemap_txt_jp_2 = fs::File::options()
        .create(true)
        .append(true)
        .open("textsearch_jp_2.xml")
        .await?;

    sitemap_vec.write(HEAD.as_bytes()).await?;
    sitemap_vec_jp.write(HEAD.as_bytes()).await?;
    sitemap_txt.write(HEAD.as_bytes()).await?;
    sitemap_txt_jp.write(HEAD.as_bytes()).await?;

    sitemap_vec_2.write(HEAD.as_bytes()).await?;
    sitemap_vec_jp_2.write(HEAD.as_bytes()).await?;
    sitemap_txt_2.write(HEAD.as_bytes()).await?;
    sitemap_txt_jp_2.write(HEAD.as_bytes()).await?;

    let mut gen_map = HashMap::new();
    let mut next = 0;
    for doc in src_list {
        if doc.item_type == 0 || doc.item_type >= 3 {
            continue;
        }

        let item_type = match doc.item_type {
            1 => "anime",
            2 => "character",
            _ => bail!("invalid item type {}", doc.item_type),
        };

        if !gen_map.contains_key(&doc.id.to_hex()) {
            let mut q = form_urlencoded::Serializer::new(String::new());
            q.append_pair("item-type", item_type);
            let url = format!("{base_url_vec}/{}?{}", doc.id, q.finish());
            let url_jp = format!("{url}&lang=ja");
            let xml = f.replace("%LOC%", &url.replace('&', "&amp;"));
            let xml_jp = f.replace("%LOC%", &url_jp.replace('&', "&amp;"));

            match next {
                0 => {
                    sitemap_vec.write(xml.as_bytes()).await?;
                    sitemap_vec_jp.write(xml_jp.as_bytes()).await?;
                }
                1 => {
                    sitemap_vec_2.write(xml.as_bytes()).await?;
                    sitemap_vec_jp_2.write(xml_jp.as_bytes()).await?;
                }
                _ => bail!("next is unexpected value"),
            }

            gen_map.insert(doc.id.to_hex(), true);
        }

        if let Some(name) = doc.name {
            if !gen_map.contains_key(&name) {
                let mut q = form_urlencoded::Serializer::new(String::new());
                q.append_pair("keyword", &name)
                    .append_pair("item-type", "all");
                let url = format!("{base_url_txt}?{}", q.finish());
                let xml = f.replace("%LOC%", &url.replace('&', "&amp;"));

                match next {
                    0 => sitemap_txt.write(xml.as_bytes()).await?,
                    1 => sitemap_txt_2.write(xml.as_bytes()).await?,
                    _ => bail!("next is unexpected value"),
                };
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

                match next {
                    0 => sitemap_txt_jp.write(xml.as_bytes()).await?,
                    1 => sitemap_txt_jp_2.write(xml.as_bytes()).await?,
                    _ => bail!("next is unexpected value"),
                };
                gen_map.insert(name_japanese, true);
            }
        };

        next = (next + 1) % 2;
    }

    sitemap_vec.write(FOOT.as_bytes()).await?;
    sitemap_vec_jp.write(FOOT.as_bytes()).await?;
    sitemap_txt.write(FOOT.as_bytes()).await?;
    sitemap_txt_jp.write(FOOT.as_bytes()).await?;

    sitemap_vec_2.write(FOOT.as_bytes()).await?;
    sitemap_vec_jp_2.write(FOOT.as_bytes()).await?;
    sitemap_txt_2.write(FOOT.as_bytes()).await?;
    sitemap_txt_jp_2.write(FOOT.as_bytes()).await?;

    sitemap_vec.flush().await?;
    sitemap_vec_jp.flush().await?;
    sitemap_txt.flush().await?;
    sitemap_txt_jp.flush().await?;

    sitemap_vec_2.flush().await?;
    sitemap_vec_jp_2.flush().await?;
    sitemap_txt_2.flush().await?;
    sitemap_txt_jp_2.flush().await?;

    info!("done");
    Ok(())
}
