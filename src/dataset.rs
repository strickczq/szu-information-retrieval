use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Dataset {
    pub docs: Vec<Doc>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Doc {
    pub url: String,
    pub infotype: String,
    pub user: String,
    pub title: String,
    pub text: String,
    pub html: String,
    pub time: String,
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Attachment {
    pub name: String,
    pub url: String,
}

impl Dataset {
    pub async fn load() -> anyhow::Result<Self> {
        tracing::info!("[Dataset] 开始读取数据");

        // recursively load ./dataset/<year>/<user>/<id>.json
        let mut docs = Vec::new();
        let mut year_dirs = tokio::fs::read_dir("./dataset").await?;
        while let Some(year_dir) = year_dirs.next_entry().await? {
            if !year_dir.file_type().await?.is_dir() {
                continue;
            }

            let mut user_dirs = tokio::fs::read_dir(year_dir.path()).await?;
            while let Some(user_dir) = user_dirs.next_entry().await? {
                if !user_dir.file_type().await?.is_dir() {
                    continue;
                }

                let mut doc_files = tokio::fs::read_dir(user_dir.path()).await?;
                while let Some(doc_file) = doc_files.next_entry().await? {
                    if !doc_file.file_type().await?.is_file() {
                        continue;
                    }

                    let doc = tokio::fs::read_to_string(doc_file.path()).await?;
                    match serde_json::from_str::<Doc>(&doc) {
                        Ok(doc) => docs.push(doc),
                        Err(_) => println!("Error doc at {:?}", doc_file.path()),
                    }
                }
            }
        }

        tracing::info!("[Dataset] 已读取 {} 个文档", docs.len());

        Ok(Self { docs })
    }
}
