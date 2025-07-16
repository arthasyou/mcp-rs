use mcp_tools_rs::tools::s3::S3Client;

#[tokio::main]
async fn main() {
    let reader = S3Client::new(
        "https://minio.cyydm.shop/",
        "admin",
        "QwerAa@112211",
        "testbucket",
    )
    .unwrap();

    let file_content = reader
        .read_text_file("https://minio.cyydm.shop/testbucket/laoyou.txt")
        .await
        .unwrap();

    println!("File Content:\n{}", file_content);
}
