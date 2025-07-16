use minio::s3::{
    client::{Client, ClientBuilder},
    creds::StaticProvider,
    http::BaseUrl,
    types::S3Api,
};

use crate::error::Result;

pub struct S3Client {
    client: Client,
}

impl S3Client {
    pub fn new(endpoint: &str, access_key: &str, secret_key: &str) -> Result<Self> {
        let base_url = endpoint.parse::<BaseUrl>()?;
        let provider = StaticProvider::new(access_key, secret_key, None);

        let client = ClientBuilder::new(base_url)
            .provider(Some(Box::new(provider)))
            .build()?;

        Ok(S3Client { client })
    }

    pub async fn read_text_file(&self, bucket: &str, file_path: &str) -> Result<String> {
        let get_object = self.client.get_object(bucket, file_path).send().await?;

        // 从 ObjectContent 读取内容
        let content = get_object.content;

        // 使用 to_segmented_bytes() 方法读取所有内容
        let segmented_bytes = content.to_segmented_bytes().await?;

        // 将 SegmentedBytes 转换为 Vec<u8>
        let mut buffer = Vec::new();
        for chunk in segmented_bytes.into_iter() {
            buffer.extend_from_slice(&chunk);
        }

        // 转换为字符串（仅适用于文本文件）
        let text_content = String::from_utf8(buffer)?;

        println!("📄 文件内容: {}", text_content);
        println!("📊 文件大小: {} bytes", get_object.object_size);

        if let Some(etag) = get_object.etag {
            println!("🏷️  ETag: {}", etag);
        }

        Ok(text_content)
    }
}
