use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

use crate::indexer::Indexer;

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

pub struct Generator {
    indexer: Indexer,
    file_idx: u32,
    file: File,
    idx: u32,
    max: u32,
    format: String,
}

impl Generator {
    async fn new_file(name: &str) -> anyhow::Result<File> {
        let mut file = fs::File::options()
            .create(true)
            .append(true)
            .open(name)
            .await?;

        file.write(HEAD.as_bytes()).await?;
        Ok(file)
    }

    pub async fn new(max: u32, lastmod: &str) -> anyhow::Result<Generator> {
        let file_idx = 0;
        let format = F.replace("%LASTMOD%", lastmod);
        let mut indexer = Indexer::new(lastmod).await?;
        let name = format!("sitemap{file_idx}.xml");
        let file = Self::new_file(&name).await?;
        indexer.write(&name).await?;

        return Ok(Generator {
            indexer,
            file_idx,
            file,
            idx: 0,
            max,
            format,
        });
    }

    pub async fn write(&mut self, src: String) -> anyhow::Result<()> {
        let xml = self.format.replace("%LOC%", &src.replace('&', "&amp;"));
        self.file.write(xml.as_bytes()).await?;
        self.idx += 1;

        if self.idx >= self.max {
            self.file.write(FOOT.as_bytes()).await?;
            self.file.flush().await?;
            self.file_idx += 1;

            let name = format!("sitemap{}.xml", self.file_idx);
            let new_file = Self::new_file(&name).await?;
            self.indexer.write(&name).await?;
            self.file = new_file;
            self.idx = 0;
        }

        Ok(())
    }

    pub async fn finish(&mut self) -> anyhow::Result<()> {
        self.file.write(FOOT.as_bytes()).await?;
        self.indexer.finish().await?;
        Ok(())
    }

    pub async fn flush(&mut self) -> anyhow::Result<()> {
        self.file.flush().await?;
        self.indexer.flush().await?;
        Ok(())
    }
}
