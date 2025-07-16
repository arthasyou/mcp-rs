use anyhow::Result;
use mcp_tools_rs::core::content;
use minio::s3::{
    client::{Client, ClientBuilder},
    creds::StaticProvider,
    http::BaseUrl,
    types::S3Api,
};
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() -> Result<()> {
    let client = create_minio_client().await?;
    println!("✅ 成功连接到 MinIO");

    // 读取文件
    read_file(&client, "testbucket", "laoyou.txt").await?;

    Ok(())
}

async fn create_minio_client() -> Result<Client> {
    let endpoint = "https://minio.cyydm.shop";
    let access_key = "admin";
    let secret_key = "QwerAa@112211";

    let base_url = endpoint.parse::<BaseUrl>()?;
    let provider = StaticProvider::new(access_key, secret_key, None);

    let client = ClientBuilder::new(base_url)
        .provider(Some(Box::new(provider)))
        .build()?;

    Ok(client)
}

async fn read_file(client: &Client, bucket: &str, object: &str) -> Result<()> {
    let get_object = client.get_object(bucket, object).send().await?;

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

    Ok(())
}
