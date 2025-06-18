use serde::{Deserialize, Serialize};

use super::{Annotation, ResourceContents, Role};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextContent {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation: Option<Annotation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageContent {
    pub data: String,
    pub mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation: Option<Annotation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddedResource {
    pub resource: ResourceContents,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotation: Option<Annotation>,
}

impl EmbeddedResource {
    pub fn get_text(&self) -> String {
        match &self.resource {
            ResourceContents::TextResourceContents { text, .. } => text.clone(),
            _ => String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Content {
    Text(TextContent),
    Image(ImageContent),
    Resource(EmbeddedResource),
}

impl Content {
    pub fn text<S: Into<String>>(text: S) -> Self {
        Content::Text(TextContent {
            text: text.into(),
            annotation: None,
        })
    }

    pub fn image<S: Into<String>, T: Into<String>>(data: S, mime_type: T) -> Self {
        Content::Image(ImageContent {
            data: data.into(),
            mime_type: mime_type.into(),
            annotation: None,
        })
    }

    pub fn resource(resource: ResourceContents) -> Self {
        Content::Resource(EmbeddedResource {
            resource,
            annotation: None,
        })
    }

    pub fn embedded_text<S: Into<String>, T: Into<String>>(uri: S, content: T) -> Self {
        Content::Resource(EmbeddedResource {
            resource: ResourceContents::TextResourceContents {
                uri: uri.into(),
                mime_type: Some("text".to_string()),
                text: content.into(),
            },
            annotation: None,
        })
    }

    /// Get the text content if this is a TextContent variant
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Content::Text(text) => Some(&text.text),
            _ => None,
        }
    }

    /// Get the image content if this is an ImageContent variant
    pub fn as_image(&self) -> Option<(&str, &str)> {
        match self {
            Content::Image(image) => Some((&image.data, &image.mime_type)),
            _ => None,
        }
    }

    /// Set the audience for the content
    pub fn with_audience(mut self, audience: Vec<Role>) -> Self {
        let annotation = match &mut self {
            Content::Text(text) => &mut text.annotation,
            Content::Image(image) => &mut image.annotation,
            Content::Resource(resource) => &mut resource.annotation,
        };
        *annotation = Some(match annotation.take() {
            Some(mut a) => {
                a.audience = Some(audience);
                a
            }
            None => Annotation {
                audience: Some(audience),
                priority: None,
                timestamp: None,
            },
        });
        self
    }

    /// Set the priority for the content
    /// # Panics
    /// Panics if priority is not between 0.0 and 1.0 inclusive
    pub fn with_priority(mut self, priority: f32) -> Self {
        if !(0.0 ..= 1.0).contains(&priority) {
            panic!("Priority must be between 0.0 and 1.0");
        }
        let annotation = match &mut self {
            Content::Text(text) => &mut text.annotation,
            Content::Image(image) => &mut image.annotation,
            Content::Resource(resource) => &mut resource.annotation,
        };
        *annotation = Some(match annotation.take() {
            Some(mut a) => {
                a.priority = Some(priority);
                a
            }
            None => Annotation {
                audience: None,
                priority: Some(priority),
                timestamp: None,
            },
        });
        self
    }

    /// Get the audience if set
    pub fn audience(&self) -> Option<&Vec<Role>> {
        match self {
            Content::Text(text) => text.annotation.as_ref().and_then(|a| a.audience.as_ref()),
            Content::Image(image) => image.annotation.as_ref().and_then(|a| a.audience.as_ref()),
            Content::Resource(resource) => resource
                .annotation
                .as_ref()
                .and_then(|a| a.audience.as_ref()),
        }
    }

    /// Get the priority if set
    pub fn priority(&self) -> Option<f32> {
        match self {
            Content::Text(text) => text.annotation.as_ref().and_then(|a| a.priority),
            Content::Image(image) => image.annotation.as_ref().and_then(|a| a.priority),
            Content::Resource(resource) => resource.annotation.as_ref().and_then(|a| a.priority),
        }
    }

    pub fn unannotated(&self) -> Self {
        match self {
            Content::Text(text) => Content::text(text.text.clone()),
            Content::Image(image) => Content::image(image.data.clone(), image.mime_type.clone()),
            Content::Resource(resource) => Content::resource(resource.resource.clone()),
        }
    }
}
