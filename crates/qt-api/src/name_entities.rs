//! DTOs for caller-supplied AI-extracted entities.
//!
//! The server makes no AI calls: clients (qt-web hoặc bất kỳ tool nào) call
//! DeepSeek/Gemini/proxy themselves with their own key, then pass the
//! extracted entities here as plain data (`aiEntities`) to be merged with the
//! rule candidates. Keys and provider endpoints never reach the server.

use serde::Deserialize;

use qt_core::NameEntityType;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiExtractedEntity {
    pub text: String,
    #[serde(default)]
    pub entity_type: Option<String>,
    #[serde(default)]
    pub suggested: Option<String>,
    #[serde(default = "default_extract_confidence")]
    pub confidence: f32,
}

fn default_extract_confidence() -> f32 {
    0.75
}

pub fn parse_entity_type(value: &str) -> NameEntityType {
    match value.to_ascii_lowercase().as_str() {
        "person" | "per" => NameEntityType::Person,
        "location" | "loc" => NameEntityType::Location,
        "organization" | "org" => NameEntityType::Organization,
        "title" => NameEntityType::Title,
        _ => NameEntityType::Unknown,
    }
}

pub fn entity_type_name(value: NameEntityType) -> &'static str {
    match value {
        NameEntityType::Person => "person",
        NameEntityType::Location => "location",
        NameEntityType::Organization => "organization",
        NameEntityType::Title => "title",
        NameEntityType::Unknown => "unknown",
    }
}
