use async_trait::async_trait;
use mcp_core_rs::{
    MimeType, Resource, Tool, content::Content, protocol::capabilities::ServerCapabilities,
};
use s3::{Bucket, Region, creds::Credentials};
use serde_json::Value;

use crate::{
    error::{Error, Result},
    server::service::{capabilities::CapabilitiesBuilder, traits::Service},
};

#[derive(Clone)]
pub struct OssService {
    bucket: Box<Bucket>,
}

impl OssService {
    pub async fn new(
        endpoint: String,
        access_key: String,
        secret_key: String,
        bucket_name: String,
    ) -> Result<Self> {
        let region = Region::Custom {
            region: "us-east-1".to_string(), // MinIO 兼容 S3 一般写 us-east-1
            endpoint,
        };

        let credentials = Credentials::new(Some(&access_key), Some(&secret_key), None, None, None)
            .map_err(|e| Error::System(format!("Credentials error: {}", e)))?;

        let bucket = Bucket::new(&bucket_name, region, credentials)
            .map_err(|e| Error::System(format!("Bucket error: {}", e)))?
            .with_path_style(); // MinIO 需要 path style

        Ok(Self { bucket })
    }

    async fn upload_file(&self, file_name: String, content: String) -> Result<String> {
        let response = self
            .bucket
            .put_object(file_name.as_str(), content.as_bytes())
            .await
            .map_err(|e| Error::System(format!("Upload error: {}", e)))?;

        if response.status_code() == 200 {
            Ok(format!("Uploaded: {}", file_name))
        } else {
            Err(Error::System(format!(
                "Upload failed, code: {}",
                response.status_code()
            )))
        }
    }

    async fn download_file(&self, file_name: String) -> Result<String> {
        let result = self
            .bucket
            .get_object(file_name.as_str())
            .await
            .map_err(|e| Error::System(format!("Download error: {}", e)))?;

        if result.status_code() == 200 {
            let text = String::from_utf8_lossy(&result.bytes()).to_string();
            Ok(text)
        } else {
            Err(Error::System(format!(
                "Download failed, code: {}",
                result.status_code()
            )))
        }
    }

    fn _create_resource_text(&self, uri: &str, name: &str) -> Resource {
        Resource::new(uri, MimeType::Text, Some(name.to_string())).unwrap()
    }
}

#[async_trait]
impl Service for OssService {
    fn name(&self) -> String {
        "oss".to_string()
    }

    fn instructions(&self) -> String {
        "This server provides OSS storage tools for uploading and downloading files.".to_string()
    }

    fn capabilities(&self) -> ServerCapabilities {
        CapabilitiesBuilder::new()
            .with_tools(false)
            .with_resources(false, false)
            .with_prompts(false)
            .build()
    }

    fn list_tools(&self) -> Vec<Tool> {
        vec![
            Tool::new(
                "upload_file".to_string(),
                "Upload a file to OSS".to_string(),
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_name": { "type": "string" },
                        "content": { "type": "string" }
                    },
                    "required": ["file_name", "content"]
                }),
            ),
            Tool::new(
                "download_file".to_string(),
                "Download a file from OSS".to_string(),
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "file_name": { "type": "string" }
                    },
                    "required": ["file_name"]
                }),
            ),
        ]
    }

    async fn call_tool(&self, tool_name: &str, arguments: Value) -> Result<Vec<Content>> {
        match tool_name {
            "upload_file" => {
                let file_name = arguments
                    .get("file_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::System("Missing file_name".to_string()))?
                    .to_string();

                let content = arguments
                    .get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::System("Missing content".to_string()))?
                    .to_string();

                let result = self.upload_file(file_name, content).await?;
                Ok(vec![Content::text(result)])
            }

            "download_file" => {
                let file_name = arguments
                    .get("file_name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Error::System("Missing file_name".to_string()))?
                    .to_string();

                let result = self.download_file(file_name).await?;
                Ok(vec![Content::text(result)])
            }

            _ => Err(Error::System(format!("Tool {} not found", tool_name))),
        }
    }

    fn list_resources(&self) -> Vec<Resource> {
        vec![self._create_resource_text("oss://bucket-info", "Bucket Info")]
    }

    async fn read_resource(&self, uri: &str) -> Result<String> {
        match uri {
            "a" => {
                // Mock response for "a"
                Ok("This is a mock response for resource 'a'.".to_string())
            }
            "b" => Ok("b".to_string()),
            _ => {
                // Mock response for any other resource
                Err(Error::System(format!("Resource {} not found", uri)))
            }
        }
        // match uri {
        //     "ossbucket-info" => {
        //         // Mock bucket info
        //         return Ok("Mock OSS bucket information".to_string());
        //     }
        //     _ => Err(Error::System(format!("Resource {} not found", uri))),
        // }
        // if uri == "oss://bucket-info" {
        //     return Ok("This is a mock OSS bucket info.".to_string());
        // }
        // Err(Error::System(format!("Resource {} not found", uri)))
    }
}
