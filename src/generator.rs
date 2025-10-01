use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};

const HEAD: &'static str = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">"#;

const FOOT: &'static str = "
</urlset>
";

pub struct Generator {
    file_idx: u32,
    file: File,
    idx: u32,
    max: u32,
}

impl Generator {
    async fn new_file(name: String) -> anyhow::Result<File> {
        let mut file = fs::File::options()
            .create(true)
            .append(true)
            .open(name)
            .await?;

        file.write(HEAD.as_bytes()).await?;
        Ok(file)
    }

    pub async fn new(max: u32) -> anyhow::Result<Generator> {
        let file_idx = 0;
        let file = Self::new_file(format!("sitemap{file_idx}.xml")).await?;

        return Ok(Generator {
            file_idx,
            file,
            idx: 0,
            max,
        });
    }

    pub async fn write(&mut self, src: String) -> anyhow::Result<()> {
        self.file.write(src.as_bytes()).await?;
        self.idx += 1;

        if self.idx >= self.max {
            self.file.write(FOOT.as_bytes()).await?;
            self.file.flush().await?;
            self.file_idx += 1;

            let new_file = Self::new_file(format!("sitemap{}.xml", self.file_idx)).await?;
            self.file = new_file;
            self.idx = 0;
        }

        Ok(())
    }

    pub async fn flush(&mut self) -> anyhow::Result<()> {
        self.file.flush().await?;
        Ok(())
    }
}
