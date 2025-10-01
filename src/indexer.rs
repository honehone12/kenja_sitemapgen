use std::env;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

const HEAD: &'static str = r#"<?xml version="1.0" encoding="UTF-8"?>
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#;

const FOOT: &'static str = "
</sitemapindex>
";

const F: &'static str = "
    <sitemap>
        <loc>%LOC%</loc>
        <lastmod>%LASTMOD%</lastmod>
    </sitemap>";

pub struct Indexer {
    file: File,
    format: String,
    base_url: String,
}

impl Indexer {
    pub async fn new(lastmod: &str) -> anyhow::Result<Indexer> {
        let mut file = fs::File::options()
            .create(true)
            .append(true)
            .open("index.xml")
            .await?;

        file.write(HEAD.as_bytes()).await?;
        let format = F.replace("%LASTMOD%", lastmod);
        let base_url = env::var("BASE_URL_SITEMAP")?;

        Ok(Indexer {
            file,
            format,
            base_url,
        })
    }

    pub async fn write(&mut self, name: &str) -> anyhow::Result<()> {
        let url = format!("{}/{name}", self.base_url);
        let xml = self.format.replace("%LOC%", &url);
        self.file.write(xml.as_bytes()).await?;
        Ok(())
    }

    pub async fn finish(&mut self) -> anyhow::Result<()> {
        self.file.write(FOOT.as_bytes()).await?;
        Ok(())
    }

    pub async fn flush(&mut self) -> anyhow::Result<()> {
        self.file.flush().await?;
        Ok(())
    }
}
