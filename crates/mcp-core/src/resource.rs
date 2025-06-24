use chrono::{DateTime, Utc};
use mcp_error_rs::{Error, Result};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::Annotation;

const EPSILON: f32 = 1e-6; // Tolerance for floating point comparison

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MimeType {
    Text,
    Blob,
}

/// Represents a resource in the extension with metadata
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    /// URI representing the resource location (e.g., "file:///path/to/file" or "str:///content")
    pub uri: String,
    /// Name of the resource
    pub name: String,
    /// Optional description of the resource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    pub mime_type: MimeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation: Option<Annotation>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase", untagged)]
pub enum ResourceContents {
    TextResourceContents {
        uri: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
        text: String,
    },
    BlobResourceContents {
        uri: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
        blob: String,
    },
}

impl Resource {
    /// Creates a new Resource from a URI with explicit mime type
    pub fn new<S: AsRef<str>>(uri: S, mime_type: MimeType, name: Option<String>) -> Result<Self> {
        let uri = uri.as_ref();
        let url = Url::parse(uri)?;

        // Extract name from the path component of the URI
        // Use provided name if available, otherwise extract from URI
        let name = match name {
            Some(n) => n,
            None => url
                .path_segments()
                .and_then(|segments| segments.last())
                .unwrap_or("unnamed")
                .to_string(),
        };

        Ok(Self {
            uri: uri.to_string(),
            name,
            description: None,
            mime_type,
            annotation: Some(Annotation::new_with_priority(0.0)),
        })
    }

    /// Creates a new Resource with explicit URI, name, and priority
    pub fn with_uri<S: Into<String>>(
        uri: S,
        name: S,
        priority: f32,
        mime_type: MimeType,
    ) -> Result<Self> {
        let uri_string: String = uri.into();
        Url::parse(&uri_string)?;

        Ok(Self {
            uri: uri_string,
            name: name.into(),
            description: None,
            mime_type,
            annotation: Some(Annotation::new_with_priority(priority)),
        })
    }

    /// Updates the resource's timestamp to the current time
    pub fn update_timestamp(&mut self) -> Result<()> {
        if let Some(ann) = &mut self.annotation {
            ann.update_timestamp();
            Ok(())
        } else {
            Err(Error::System("Missing annotation".into()))
        }
    }

    // TODO 所有和annotiaon的方法设计不好需要修改

    /// Sets the priority of the resource and returns self for method chaining
    pub fn with_priority(mut self, priority: f32) -> Self {
        self.annotation.as_mut().unwrap().priority = Some(priority);
        self
    }

    /// Mark the resource as active, i.e. set its priority to 1.0
    pub fn mark_active(self) -> Self {
        self.with_priority(1.0)
    }

    // Check if the resource is active
    pub fn is_active(&self) -> bool {
        if let Some(priority) = self.priority() {
            (priority - 1.0).abs() < EPSILON
        } else {
            false
        }
    }

    /// Returns the priority of the resource, if set
    pub fn priority(&self) -> Option<f32> {
        self.annotation.as_ref().and_then(|a| a.priority)
    }

    /// Returns the timestamp of the resource, if set
    pub fn timestamp(&self) -> Option<DateTime<Utc>> {
        self.annotation.as_ref().and_then(|a| a.timestamp)
    }

    /// Returns the scheme of the URI
    pub fn scheme(&self) -> Result<String> {
        let url = Url::parse(&self.uri)?;
        Ok(url.scheme().to_string())
    }

    /// Sets the description of the resource
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the MIME type of the resource
    pub fn with_mime_type(mut self, mime_type: MimeType) -> Self {
        self.mime_type = mime_type;
        self
    }
}
