//! Widget schema definitions for agent discoverability.

use serde::{Deserialize, Serialize};

/// Describes the schema of a widget type for agent discoverability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSchema {
    /// Unique type name (e.g., "Button", "TextInput", "Panel").
    pub name: String,
    /// Human-readable description of the widget's purpose.
    pub description: String,
    /// The semantic role this widget type typically plays.
    pub default_role: SemanticRole,
    /// Properties that can be configured on this widget.
    pub properties: Vec<PropertySchema>,
    /// Actions that agents can invoke on this widget type.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<super::AgentAction>,
    /// Brief usage example for agents.
    pub usage_hint: Option<String>,
    /// Tags for fuzzy search (e.g., ["button", "action", "click"]).
    pub tags: Vec<String>,
}

/// Describes a single configurable property on a widget.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PropertySchema {
    /// Property name (e.g., "text", "style", "enabled").
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// The type of this property.
    pub property_type: PropertyType,
    /// Whether this property is required.
    pub required: bool,
    /// Default value as JSON.
    pub default_value: Option<serde_json::Value>,
    /// Optional constraints on the property value.
    pub constraints: Vec<PropertyConstraint>,
}

/// Type of a widget property.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PropertyType {
    String,
    Integer,
    Float,
    Boolean,
    Color,
    Style,
    Rect,
    Size,
    Position,
    Enum(Vec<String>),
    Array(Box<PropertyType>),
    Object(Vec<PropertySchema>),
    Widget,
    Any,
}

/// A constraint on a property value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PropertyConstraint {
    Min(f64),
    Max(f64),
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),
    OneOf(Vec<serde_json::Value>),
}

/// The semantic role a widget plays in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SemanticRole {
    /// Displays text or data to the user (read-only).
    Display,
    /// Accepts user input (text fields, checkboxes, sliders, etc.).
    Input,
    /// Provides navigation between views or sections.
    Navigation,
    /// Contains and arranges other widgets.
    Container,
    /// Indicates progress or loading state.
    Progress,
    /// Displays a selection from a set of options.
    Selection,
    /// Provides interactive data visualization.
    DataVisualization,
    /// A decorative or structural element (borders, spacers).
    Decoration,
    /// An interactive button or action trigger.
    Action,
    /// A scrollable region.
    Scrollable,
    /// A modal dialog or overlay.
    Modal,
    /// A menu or menu item.
    Menu,
    /// A toolbar or toolbar item.
    Toolbar,
    /// A tab or tab bar.
    Tab,
    /// A tree node in a hierarchical view.
    TreeNode,
    /// A canvas or drawing area.
    Canvas,
    /// A media element (image, video, audio).
    Media,
    /// A system-level service (i18n, plugins, window management, theme).
    System,
    /// A diagnostic or metrics component (profiling, performance).
    Diagnostic,
    /// A platform integration component (dialogs, tray, file system).
    Configuration,
    /// An unknown or custom role.
    Custom,
}

impl WidgetSchema {
    /// Create a minimal widget schema.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        role: SemanticRole,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            default_role: role,
            properties: vec![],
            actions: vec![],
            usage_hint: None,
            tags: vec![],
        }
    }
}

impl PropertySchema {
    /// Create a simple property schema.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        property_type: PropertyType,
        required: bool,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            property_type,
            required,
            default_value: None,
            constraints: vec![],
        }
    }
}

impl std::fmt::Display for SemanticRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Display => "display",
            Self::Input => "input",
            Self::Navigation => "navigation",
            Self::Container => "container",
            Self::Progress => "progress",
            Self::Selection => "selection",
            Self::DataVisualization => "data-visualization",
            Self::Decoration => "decoration",
            Self::Action => "action",
            Self::Scrollable => "scrollable",
            Self::Modal => "modal",
            Self::Menu => "menu",
            Self::Toolbar => "toolbar",
            Self::Tab => "tab",
            Self::TreeNode => "tree-node",
            Self::Canvas => "canvas",
            Self::Media => "media",
            Self::System => "system",
            Self::Diagnostic => "diagnostic",
            Self::Configuration => "configuration",
            Self::Custom => "custom",
        };
        f.write_str(s)
    }
}
