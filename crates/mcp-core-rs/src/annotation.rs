use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::Role;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Annotation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<Vec<Role>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
}

impl Annotation {
    /// Create annotations with a given priority and the current timestamp.
    pub fn new_with_priority(priority: f32) -> Self {
        assert!(
            (0.0 ..= 1.0).contains(&priority),
            "Priority {priority} must be between 0.0 and 1.0"
        );
        Self {
            priority: Some(priority),
            timestamp: Some(Utc::now()),
            audience: None,
        }
    }

    /// Create annotations with a given audience and the current timestamp.
    pub fn new_with_audience(audience: Vec<Role>) -> Self {
        Self {
            audience: Some(audience),
            priority: None,
            timestamp: Some(Utc::now()),
        }
    }

    /// Create annotations with a given priority and audience, and the current timestamp.
    pub fn new_with_priority_and_audience(priority: f32, audience: Vec<Role>) -> Self {
        assert!(
            (0.0 ..= 1.0).contains(&priority),
            "Priority {priority} must be between 0.0 and 1.0"
        );
        Self {
            audience: Some(audience),
            priority: Some(priority),
            timestamp: Some(Utc::now()),
        }
    }

    pub fn set_priority(&mut self, priority: f32) {
        assert!(
            (0.0 ..= 1.0).contains(&priority),
            "Priority must be between 0.0 and 1.0"
        );
        self.priority = Some(priority);
    }

    /// 设置新的受众角色
    pub fn set_audience(&mut self, audience: Vec<Role>) {
        self.audience = Some(audience);
    }

    /// 更新时间戳为当前时间
    pub fn update_timestamp(&mut self) {
        self.timestamp = Some(Utc::now());
    }
}

impl Default for Annotation {
    fn default() -> Self {
        Self::new_with_priority(0.0)
    }
}
