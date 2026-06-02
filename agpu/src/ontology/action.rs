//! Agent actions and parameter types for widget interaction.

use serde::{Deserialize, Serialize};

/// An action that an agent can invoke on a widget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAction {
    /// Unique action name (e.g., "click", "set_text", "scroll_to").
    pub name: String,
    /// Human-readable description of what this action does.
    pub description: String,
    /// Parameters this action accepts.
    pub params: Vec<ActionParam>,
    /// Description of the return value.
    pub returns: Option<String>,
    /// Whether this action mutates the widget state.
    pub mutates: bool,
    /// Whether this action is idempotent (safe to retry).
    pub idempotent: bool,
    /// Keyboard shortcut, if any (e.g., "Ctrl+S").
    pub shortcut: Option<String>,
}

/// A parameter for an agent action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionParam {
    /// Parameter name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// The type of this parameter.
    pub param_type: ActionParamType,
    /// Whether this parameter is required.
    pub required: bool,
    /// Default value as JSON, if optional.
    pub default_value: Option<serde_json::Value>,
}

/// Type of an action parameter.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionParamType {
    String,
    Integer,
    Float,
    Boolean,
    Index,
    Position { x: bool, y: bool },
    Size { width: bool, height: bool },
    Color,
    Enum(Vec<String>),
    Any,
}

impl AgentAction {
    /// Create a simple action with no parameters.
    #[must_use]
    pub fn simple(name: impl Into<String>, description: impl Into<String>, mutates: bool) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            params: vec![],
            returns: None,
            mutates,
            idempotent: !mutates,
            shortcut: None,
        }
    }

    /// Create an action with the given parameters.
    #[must_use]
    pub fn with_params(
        name: impl Into<String>,
        description: impl Into<String>,
        params: Vec<ActionParam>,
        mutates: bool,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            params,
            returns: None,
            mutates,
            idempotent: !mutates,
            shortcut: None,
        }
    }

    /// Validate a JSON params object against this action's declared parameters.
    pub fn validate_params(&self, params: &serde_json::Value) -> Result<(), String> {
        for param in &self.params {
            let val = params.get(&param.name);
            match val {
                None | Some(serde_json::Value::Null) => {
                    if param.required {
                        return Err(format!("Missing required parameter '{}'", param.name));
                    }
                }
                Some(v) => {
                    param
                        .param_type
                        .check(v)
                        .map_err(|e| format!("Parameter '{}': {}", param.name, e))?;
                }
            }
        }
        Ok(())
    }
}

impl ActionParamType {
    fn check(&self, value: &serde_json::Value) -> Result<(), String> {
        match self {
            ActionParamType::String => {
                if !value.is_string() {
                    return Err(format!("expected string, got {}", json_type_name(value)));
                }
            }
            ActionParamType::Integer => {
                if !value.is_i64() && !value.is_u64() {
                    return Err(format!("expected integer, got {}", json_type_name(value)));
                }
            }
            ActionParamType::Float => {
                if !value.is_number() {
                    return Err(format!("expected number, got {}", json_type_name(value)));
                }
            }
            ActionParamType::Boolean => {
                if !value.is_boolean() {
                    return Err(format!("expected boolean, got {}", json_type_name(value)));
                }
            }
            ActionParamType::Index => {
                if !value.is_u64() {
                    return Err(format!(
                        "expected index (uint), got {}",
                        json_type_name(value)
                    ));
                }
            }
            ActionParamType::Position { .. } | ActionParamType::Size { .. } => {
                if !value.is_object() {
                    return Err(format!("expected object, got {}", json_type_name(value)));
                }
            }
            ActionParamType::Color => {
                if !value.is_string() && !value.is_object() {
                    return Err(format!(
                        "expected color string or object, got {}",
                        json_type_name(value)
                    ));
                }
            }
            ActionParamType::Enum(variants) => {
                if let Some(s) = value.as_str() {
                    if !variants.iter().any(|v| v == s) {
                        return Err(format!("expected one of {:?}, got {:?}", variants, s));
                    }
                } else {
                    return Err(format!(
                        "expected string enum, got {}",
                        json_type_name(value)
                    ));
                }
            }
            ActionParamType::Any => {}
        }
        Ok(())
    }
}

impl ActionParam {
    /// Create a required parameter.
    #[must_use]
    pub fn required(
        name: impl Into<String>,
        description: impl Into<String>,
        param_type: ActionParamType,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            param_type,
            required: true,
            default_value: None,
        }
    }

    /// Create an optional parameter with a default.
    #[must_use]
    pub fn optional(
        name: impl Into<String>,
        description: impl Into<String>,
        param_type: ActionParamType,
        default: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            param_type,
            required: false,
            default_value: Some(default),
        }
    }
}

fn json_type_name(v: &serde_json::Value) -> &'static str {
    match v {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_action_defaults() {
        let a = AgentAction::simple("click", "Click the button", true);
        assert_eq!(a.name, "click");
        assert!(a.mutates);
        assert!(!a.idempotent);
        assert!(a.params.is_empty());
        assert!(a.shortcut.is_none());
    }

    #[test]
    fn simple_action_readonly_is_idempotent() {
        let a = AgentAction::simple("read", "Read value", false);
        assert!(!a.mutates);
        assert!(a.idempotent);
    }

    #[test]
    fn validate_params_missing_required() {
        let action = AgentAction::with_params(
            "set_text",
            "Set text content",
            vec![ActionParam::required(
                "text",
                "The text",
                ActionParamType::String,
            )],
            true,
        );
        let result = action.validate_params(&serde_json::json!({}));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Missing required"));
    }

    #[test]
    fn validate_params_wrong_type() {
        let action = AgentAction::with_params(
            "set_value",
            "Set value",
            vec![ActionParam::required(
                "count",
                "Count",
                ActionParamType::Integer,
            )],
            true,
        );
        let result = action.validate_params(&serde_json::json!({"count": "not_a_number"}));
        assert!(result.is_err());
    }

    #[test]
    fn validate_params_correct() {
        let action = AgentAction::with_params(
            "set_value",
            "Set value",
            vec![ActionParam::required(
                "count",
                "Count",
                ActionParamType::Integer,
            )],
            true,
        );
        assert!(
            action
                .validate_params(&serde_json::json!({"count": 42}))
                .is_ok()
        );
    }

    #[test]
    fn validate_params_optional_missing_ok() {
        let action = AgentAction::with_params(
            "set",
            "Set",
            vec![ActionParam::optional(
                "label",
                "Label",
                ActionParamType::String,
                serde_json::json!("default"),
            )],
            true,
        );
        assert!(action.validate_params(&serde_json::json!({})).is_ok());
    }

    #[test]
    fn validate_boolean_type() {
        let action = AgentAction::with_params(
            "toggle",
            "Toggle",
            vec![ActionParam::required(
                "state",
                "State",
                ActionParamType::Boolean,
            )],
            true,
        );
        assert!(
            action
                .validate_params(&serde_json::json!({"state": true}))
                .is_ok()
        );
        assert!(
            action
                .validate_params(&serde_json::json!({"state": "yes"}))
                .is_err()
        );
    }

    #[test]
    fn validate_enum_type() {
        let action = AgentAction::with_params(
            "set_mode",
            "Set mode",
            vec![ActionParam::required(
                "mode",
                "Mode",
                ActionParamType::Enum(vec!["light".into(), "dark".into()]),
            )],
            true,
        );
        assert!(
            action
                .validate_params(&serde_json::json!({"mode": "dark"}))
                .is_ok()
        );
        assert!(
            action
                .validate_params(&serde_json::json!({"mode": "blue"}))
                .is_err()
        );
    }

    #[test]
    fn validate_any_type_accepts_anything() {
        let action = AgentAction::with_params(
            "exec",
            "Execute",
            vec![ActionParam::required("data", "Data", ActionParamType::Any)],
            true,
        );
        assert!(
            action
                .validate_params(&serde_json::json!({"data": [1,2,3]}))
                .is_ok()
        );
        assert!(
            action
                .validate_params(&serde_json::json!({"data": "text"}))
                .is_ok()
        );
        assert!(
            action
                .validate_params(&serde_json::json!({"data": 42}))
                .is_ok()
        );
    }

    #[test]
    fn action_param_required_constructor() {
        let p = ActionParam::required("name", "The name", ActionParamType::String);
        assert!(p.required);
        assert!(p.default_value.is_none());
    }

    #[test]
    fn action_param_optional_constructor() {
        let p = ActionParam::optional(
            "name",
            "The name",
            ActionParamType::String,
            serde_json::json!(""),
        );
        assert!(!p.required);
        assert!(p.default_value.is_some());
    }
}
