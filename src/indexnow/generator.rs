use tokio::fs;

use crate::indexnow::model::Request;

pub struct RequestGenerator {
    file_idx: u32,
    idx: u32,
    max: u32,
    host: String,
    key: String,
    key_location: String,
    buffer: Vec<String>,
}

impl RequestGenerator {
    pub fn new(max: u32, host: &str, key: &str, key_location: &str) -> Self {
        return RequestGenerator {
            file_idx: 0,
            idx: 0,
            max,
            host: host.to_string(),
            key: key.to_string(),
            key_location: key_location.to_string(),
            buffer: vec![],
        };
    }

    async fn write(&mut self) -> anyhow::Result<()> {
        let buff = std::mem::take(&mut self.buffer);
        let req = Request {
            host: self.host.clone(),
            key: self.key.clone(),
            key_location: self.key_location.clone(),
            url_list: buff,
        };
        let name = format!("indexnow{}.json", self.file_idx);
        let b = serde_json::to_string(&req)?;
        fs::write(name, b.as_bytes()).await?;
        Ok(())
    }

    pub async fn push(&mut self, src: &str) -> anyhow::Result<()> {
        self.buffer.push(src.to_string());
        self.idx += 1;

        if self.idx > self.max {
            self.write().await?;
            self.file_idx += 1;
            self.idx = 0;
        }

        Ok(())
    }

    pub async fn finish(&mut self) -> anyhow::Result<()> {
        self.write().await?;
        Ok(())
    }
}
