/// Roles to describe the origin/ownership of content
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

impl Default for Role {
    fn default() -> Self {
        Role::User
    }
}

impl Role {
    /// Returns all roles as a vector
    pub fn all() -> Vec<Self> {
        vec![Role::User, Role::Assistant]
    }
    /// Returns the user role
    pub fn user() -> Self {
        Role::User
    }
    /// Returns the assistant role
    pub fn assistant() -> Self {
        Role::Assistant
    }

    /// Returns the name of the role as a static string
    pub fn get_name(&self) -> &'static str {
        match self {
            Role::User => "user",
            Role::Assistant => "assistant",
        }
    }
}
