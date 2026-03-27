use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionContributes {
    #[serde(default)]
    pub commands: Vec<CommandContribution>,
    #[serde(default)]
    pub languages: Vec<LanguageContribution>,
    #[serde(default)]
    pub grammars: Vec<GrammarContribution>,
    #[serde(default)]
    pub themes: Vec<ThemeContribution>,
    #[serde(default)]
    pub icon_themes: Vec<IconThemeContribution>,
    #[serde(default)]
    pub keybindings: Vec<KeybindingContribution>,
    #[serde(default)]
    pub menus: HashMap<String, Vec<MenuItemContribution>>,
    #[serde(default)]
    pub snippets: Vec<SnippetContribution>,
    #[serde(default)]
    pub configuration: Option<ConfigurationContribution>,
    #[serde(default)]
    pub configuration_defaults: Option<Value>,
    #[serde(default)]
    pub debuggers: Vec<DebuggerContribution>,
    #[serde(default)]
    pub breakpoints: Vec<BreakpointContribution>,
    #[serde(default)]
    pub views: HashMap<String, Vec<ViewContribution>>,
    #[serde(default)]
    pub views_welcome: Vec<ViewsWelcomeContribution>,
    #[serde(default)]
    pub terminal_profiles: Vec<TerminalProfileContribution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandContribution {
    pub command: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub icon: Option<IconDefinition>,
    #[serde(default)]
    pub enablement: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageContribution {
    pub id: String,
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub filenames: Vec<String>,
    #[serde(default)]
    pub filename_patterns: Vec<String>,
    #[serde(default)]
    pub first_line: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub configuration: Option<String>,
    #[serde(default)]
    pub icon: Option<IconDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrammarContribution {
    pub language: String,
    pub scope_name: String,
    pub path: String,
    #[serde(default)]
    pub embedded_languages: HashMap<String, String>,
    #[serde(default)]
    pub token_types: HashMap<String, String>,
    #[serde(default)]
    pub token_modifiers: HashMap<String, String>,
    #[serde(default)]
    pub inject_to: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeContribution {
    pub label: String,
    pub ui_theme: Option<String>,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconThemeContribution {
    pub id: String,
    pub label: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeybindingContribution {
    pub key: String,
    #[serde(default)]
    pub command: String,
    #[serde(default)]
    pub when: Option<String>,
    #[serde(default)]
    pub args: Option<Value>,
    #[serde(default)]
    pub mac: Option<String>,
    #[serde(default)]
    pub linux: Option<String>,
    #[serde(default)]
    pub win: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuItemContribution {
    pub command: String,
    #[serde(default)]
    pub when: Option<String>,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub alt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnippetContribution {
    pub language: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationContribution {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub properties: HashMap<String, ConfigurationProperty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigurationProperty {
    #[serde(rename = "type")]
    pub prop_type: Option<String>,
    #[serde(default)]
    pub default: Option<Value>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub enum_values: Option<Vec<Value>>,
    #[serde(default)]
    pub enum_descriptions: Option<Vec<String>>,
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
            if let Some(ref when) = view_welcome.when {
                events.push(format!("onView:{}", view_welcome.view));
            }
        }

        events
    }
}
