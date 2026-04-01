//! Extension contribution types parsed from `package.json` `contributes` fields.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// All contributions declared in an extension's `package.json`.
///
/// Each field corresponds to a top-level key under the `contributes` object
/// and defaults to an empty collection when absent.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionContributes {
    /// Commands the extension registers in the command palette.
    #[serde(default)]
    pub commands: Vec<CommandContribution>,
    /// Language identifiers and file-association rules.
    #[serde(default)]
    pub languages: Vec<LanguageContribution>,
    /// TextMate grammars for syntax highlighting.
    #[serde(default)]
    pub grammars: Vec<GrammarContribution>,
    /// Color themes the extension provides.
    #[serde(default)]
    pub themes: Vec<ThemeContribution>,
    /// File icon themes the extension provides.
    #[serde(default)]
    pub icon_themes: Vec<IconThemeContribution>,
    /// Keyboard shortcut overrides contributed by the extension.
    #[serde(default)]
    pub keybindings: Vec<KeybindingContribution>,
    /// Menu items contributed to specific menu contexts.
    #[serde(default)]
    pub menus: HashMap<String, Vec<MenuItemContribution>>,
    /// Code snippet files contributed by the extension.
    #[serde(default)]
    pub snippets: Vec<SnippetContribution>,
    /// Schema of configuration settings the extension exposes.
    #[serde(default)]
    pub configuration: Option<ConfigurationContribution>,
    /// Default values for configuration settings.
    #[serde(default)]
    pub configuration_defaults: Option<Value>,
    /// Debug adapter contributions.
    #[serde(default)]
    pub debuggers: Vec<DebuggerContribution>,
    /// Breakpoint-type contributions per language.
    #[serde(default)]
    pub breakpoints: Vec<BreakpointContribution>,
    /// View containers and their children, keyed by view location.
    #[serde(default)]
    pub views: HashMap<String, Vec<ViewContribution>>,
    /// Welcome content shown inside view panels.
    #[serde(default)]
    pub views_welcome: Vec<ViewsWelcomeContribution>,
    /// Terminal profiles the extension contributes.
    #[serde(default)]
    pub terminal_profiles: Vec<TerminalProfileContribution>,
}

/// A command contributed to the command palette.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandContribution {
    /// Fully-qualified command identifier (e.g. `myExt.helloWorld`).
    pub command: String,
    /// Human-readable title shown in the palette.
    #[serde(default)]
    pub title: String,
    /// Optional category grouping (e.g. `"My Extension"`).
    #[serde(default)]
    pub category: Option<String>,
    /// Optional icon (light/dark variants) for the command.
    #[serde(default)]
    pub icon: Option<IconDefinition>,
    /// Optional enablement condition expression (e.g. `"editorFocus"`).
    #[serde(default)]
    pub enablement: Option<String>,
}

/// A language identifier and its file-association rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageContribution {
    /// Language identifier (e.g. `"rust"`, `"typescript"`).
    pub id: String,
    /// File extensions that map to this language (e.g. `[".rs"]`).
    #[serde(default)]
    pub extensions: Vec<String>,
    /// Exact filenames that map to this language (e.g. `["Makefile"]`).
    #[serde(default)]
    pub filenames: Vec<String>,
    /// Glob patterns for filenames that map to this language.
    #[serde(default)]
    pub filename_patterns: Vec<String>,
    /// Regex matched against the first line of a file to detect the language.
    #[serde(default)]
    pub first_line: Option<String>,
    /// User-facing aliases for the language.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Path to a TextMate language configuration JSON file.
    #[serde(default)]
    pub configuration: Option<String>,
    /// Optional icon (light/dark variants) for files of this language.
    #[serde(default)]
    pub icon: Option<IconDefinition>,
}

/// A TextMate grammar contribution for syntax highlighting.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrammarContribution {
    /// Language identifier this grammar targets.
    pub language: String,
    /// Top-level TextMate scope name (e.g. `source.rust`).
    pub scope_name: String,
    /// Relative path to the grammar file (`.tmLanguage.json` or `.plist`).
    pub path: String,
    /// Embedded language scope-name-to-id mappings.
    #[serde(default)]
    pub embedded_languages: HashMap<String, String>,
    /// Custom token type mappings.
    #[serde(default)]
    pub token_types: HashMap<String, String>,
    /// Custom token modifier mappings.
    #[serde(default)]
    pub token_modifiers: HashMap<String, String>,
    /// Scope names into which this grammar should be injected.
    #[serde(default)]
    pub inject_to: Vec<String>,
}

/// A color theme contribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeContribution {
    /// Human-readable theme label.
    pub label: String,
    /// Base UI theme identifier (e.g. `"vs-dark"`).
    pub ui_theme: Option<String>,
    /// Relative path to the theme JSON file.
    pub path: String,
}

/// A file icon theme contribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconThemeContribution {
    /// Unique identifier for the icon theme.
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// Relative path to the icon theme definition file.
    pub path: String,
}

/// A keyboard shortcut contribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeybindingContribution {
    /// Default keybinding (e.g. `"ctrl+shift+p"`).
    pub key: String,
    /// Command identifier to invoke.
    #[serde(default)]
    pub command: String,
    /// Optional condition expression controlling when the binding is active.
    #[serde(default)]
    pub when: Option<String>,
    /// Optional arguments to pass to the command.
    #[serde(default)]
    pub args: Option<Value>,
    /// macOS-specific keybinding override.
    #[serde(default)]
    pub mac: Option<String>,
    /// Linux-specific keybinding override.
    #[serde(default)]
    pub linux: Option<String>,
    /// Windows-specific keybinding override.
    #[serde(default)]
    pub win: Option<String>,
}

/// A menu item placed in a specific menu context.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuItemContribution {
    /// Command identifier to invoke when the menu item is selected.
    pub command: String,
    /// Optional condition expression controlling menu item visibility.
    #[serde(default)]
    pub when: Option<String>,
    /// Optional sorting group within the menu.
    #[serde(default)]
    pub group: Option<String>,
    /// Alternative command identifier for secondary-click behavior.
    #[serde(default)]
    pub alt: Option<String>,
}

/// A code snippet file contribution scoped to a language.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnippetContribution {
    /// Language identifier the snippets apply to.
    pub language: String,
    /// Relative path to the snippet definition file.
    pub path: String,
}

/// Schema of configuration settings the extension exposes to the IDE.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationContribution {
    /// Human-readable title for this configuration section.
    #[serde(default)]
    pub title: String,
    /// Property name to [`ConfigurationProperty`] map.
    #[serde(default)]
    pub properties: HashMap<String, ConfigurationProperty>,
}

/// A single configuration property declared by an extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationProperty {
    /// JSON Schema type (e.g. `"string"`, `"number"`, `"boolean"`).
    #[serde(rename = "type")]
    pub prop_type: Option<String>,
    /// Default value for the property.
    #[serde(default)]
    pub default: Option<Value>,
    /// Human-readable description of the property.
    #[serde(default)]
    pub description: Option<String>,
    /// Allowed enum values.
    #[serde(default)]
    pub enum_values: Option<Vec<Value>>,
    /// Descriptions corresponding to each enum value.
    #[serde(default)]
    pub enum_descriptions: Option<Vec<String>>,
    /// Scope of the setting (`"application"`, `"machine"`, `"window"`, `"resource"`).
    #[serde(default)]
    pub scope: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DebuggerContribution {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub program: Option<String>,
    #[serde(default)]
    pub runtime: Option<String>,
    #[serde(default)]
    pub runtime_args: Option<Vec<String>>,
    #[serde(default)]
    pub languages: Option<Vec<String>>,
    #[serde(default)]
    pub configuration_attributes: Option<Value>,
    #[serde(default)]
    pub initial_configurations: Option<Value>,
    #[serde(default)]
    pub configuration_snippets: Option<Value>,
    #[serde(default)]
    pub variables: Option<HashMap<String, String>>,
    #[serde(default)]
    pub adapter_executable_command: Option<String>,
    #[serde(default)]
    pub start_session_command: Option<String>,
    #[serde(default)]
    pub terminate_command: Option<String>,
    #[serde(default)]
    pub kill_command: Option<String>,
    #[serde(default)]
    pub restart_command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BreakpointContribution {
    #[serde(default)]
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewContribution {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub when: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewsWelcomeContribution {
    #[serde(default)]
    pub view: String,
    #[serde(default)]
    pub contents: String,
    #[serde(default)]
    pub when: Option<String>,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub enablement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TerminalProfileContribution {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub when: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconDefinition {
    #[serde(default)]
    pub light: Option<String>,
    #[serde(default)]
    pub dark: Option<String>,
}

impl ExtensionContributes {
    pub fn from_json(value: &Value) -> Self {
        if value.is_null() {
            return Self::default();
        }
        serde_json::from_value(value.clone()).unwrap_or_default()
    }

    pub fn all_command_ids(&self) -> Vec<String> {
        self.commands.iter().map(|c| c.command.clone()).collect()
    }

    pub fn language_ids_for_extension(&self, file_ext: &str) -> Vec<String> {
        let ext =
            if file_ext.starts_with('.') { file_ext.to_string() } else { format!(".{}", file_ext) };
        self.languages
            .iter()
            .filter(|l| {
                l.extensions.iter().any(|e| {
                    format!(".{}", e.trim_start_matches('.')) == ext
                        || e == file_ext.trim_start_matches('.')
                })
            })
            .map(|l| l.id.clone())
            .collect()
    }

    pub fn activation_events(&self) -> Vec<String> {
        let mut events = Vec::new();

        for cmd in &self.commands {
            events.push(format!("onCommand:{}", cmd.command));
        }

        for lang in &self.languages {
            events.push(format!("onLanguage:{}", lang.id));
        }

        for view_welcome in &self.views_welcome {
            if view_welcome.when.is_some() {
                events.push(format!("onView:{}", view_welcome.view));
            }
        }

        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_extension_contributes_default() {
        let ec = ExtensionContributes::default();
        assert!(ec.commands.is_empty());
        assert!(ec.languages.is_empty());
        assert!(ec.grammars.is_empty());
        assert!(ec.themes.is_empty());
        assert!(ec.icon_themes.is_empty());
        assert!(ec.keybindings.is_empty());
        assert!(ec.menus.is_empty());
        assert!(ec.snippets.is_empty());
        assert!(ec.configuration.is_none());
        assert!(ec.configuration_defaults.is_none());
        assert!(ec.debuggers.is_empty());
        assert!(ec.breakpoints.is_empty());
        assert!(ec.views.is_empty());
        assert!(ec.views_welcome.is_empty());
        assert!(ec.terminal_profiles.is_empty());
    }

    #[test]
    fn test_extension_contributes_from_json_null() {
        let ec = ExtensionContributes::from_json(&Value::Null);
        assert!(ec.commands.is_empty());
    }

    #[test]
    fn test_extension_contributes_from_json_with_commands() {
        let value = json!({
            "commands": [
                { "command": "myExt.hello", "title": "Say Hello" }
            ]
        });
        let ec = ExtensionContributes::from_json(&value);
        assert_eq!(ec.commands.len(), 1);
        assert_eq!(ec.commands[0].command, "myExt.hello");
        assert_eq!(ec.commands[0].title, "Say Hello");
    }

    #[test]
    fn test_extension_contributes_from_json_with_languages() {
        let value = json!({
            "languages": [
                { "id": "rust", "extensions": [".rs"] }
            ]
        });
        let ec = ExtensionContributes::from_json(&value);
        assert_eq!(ec.languages.len(), 1);
        assert_eq!(ec.languages[0].id, "rust");
        assert_eq!(ec.languages[0].extensions, vec![".rs"]);
    }

    #[test]
    fn test_extension_contributes_from_json_invalid() {
        // Not an object - should return default
        let ec = ExtensionContributes::from_json(&json!("not an object"));
        assert!(ec.commands.is_empty());
    }

    #[test]
    fn test_extension_contributes_all_command_ids() {
        let mut ec = ExtensionContributes::default();
        ec.commands.push(CommandContribution {
            command: "ext.cmd1".to_string(),
            title: "Cmd 1".to_string(),
            category: None,
            icon: None,
            enablement: None,
        });
        ec.commands.push(CommandContribution {
            command: "ext.cmd2".to_string(),
            title: "Cmd 2".to_string(),
            category: None,
            icon: None,
            enablement: None,
        });
        let ids = ec.all_command_ids();
        assert_eq!(ids, vec!["ext.cmd1", "ext.cmd2"]);
    }

    #[test]
    fn test_extension_contributes_activation_events() {
        let mut ec = ExtensionContributes::default();
        ec.commands.push(CommandContribution {
            command: "ext.run".to_string(),
            title: "Run".to_string(),
            category: None,
            icon: None,
            enablement: None,
        });
        ec.languages.push(LanguageContribution {
            id: "rust".to_string(),
            extensions: vec![],
            filenames: vec![],
            filename_patterns: vec![],
            first_line: None,
            aliases: vec![],
            configuration: None,
            icon: None,
        });
        ec.views_welcome.push(ViewsWelcomeContribution {
            view: "explorer".to_string(),
            contents: String::new(),
            when: Some("true".to_string()),
            group: None,
            enablement: None,
        });
        let events = ec.activation_events();
        assert!(events.contains(&"onCommand:ext.run".to_string()));
        assert!(events.contains(&"onLanguage:rust".to_string()));
        assert!(events.contains(&"onView:explorer".to_string()));
    }

    #[test]
    fn test_extension_contributes_language_ids_for_extension() {
        let mut ec = ExtensionContributes::default();
        ec.languages.push(LanguageContribution {
            id: "rust".to_string(),
            extensions: vec![".rs".to_string()],
            filenames: vec![],
            filename_patterns: vec![],
            first_line: None,
            aliases: vec![],
            configuration: None,
            icon: None,
        });
        let ids = ec.language_ids_for_extension(".rs");
        assert_eq!(ids, vec!["rust"]);
    }

    #[test]
    fn test_extension_contributes_language_ids_for_extension_without_dot() {
        let mut ec = ExtensionContributes::default();
        ec.languages.push(LanguageContribution {
            id: "python".to_string(),
            extensions: vec!["py".to_string()],
            filenames: vec![],
            filename_patterns: vec![],
            first_line: None,
            aliases: vec![],
            configuration: None,
            icon: None,
        });
        let ids = ec.language_ids_for_extension("py");
        assert_eq!(ids, vec!["python"]);
        // Also with leading dot
        let ids2 = ec.language_ids_for_extension(".py");
        assert_eq!(ids2, vec!["python"]);
    }
}