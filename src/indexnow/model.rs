use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Request {
    pub host: String,
    pub key: String,
    #[serde(rename = "keyLocation")]
    pub key_location: String,
    #[serde(rename = "urlList")]
    pub url_list: Vec<String>,
}
