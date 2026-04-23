//! Document tracking, text editing, and decoration management.
//!
//! This module contains document-related operations that were part of
//! SessionManager in the monolithic codebase. In the standalone crate,
//! these are provided as functions on the [`DocumentManager`] struct.

use regex::Regex;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Position in a text document (line/character).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PositionPayload {
    pub line: usize,
    pub character: usize,
}

/// Range in a text document (start/end positions).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RangePayload {
    pub start: PositionPayload,
    pub end: PositionPayload,
}

impl RangePayload {
    pub fn from_position(pos: PositionPayload) -> Self {
        Self {
            start: pos.clone(),
            end: pos,
        }
    }
}

/// Text edit payload.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct TextEditPayload {
    pub range: RangePayload,
    #[serde(rename = "newText")]
    pub new_text: String,
}

/// Manages document versions, decorations, and content operations.
pub struct DocumentManager {
    pub(crate) document_versions: Arc<RwLock<HashMap<String, i32>>>,
    pub(crate) editor_decorations: Arc<RwLock<HashMap<String, HashMap<String, Value>>>>,
    pub(crate) decoration_types: Arc<RwLock<HashMap<String, Value>>>,
}

impl DocumentManager {
    /// Create a new document manager.
    pub fn new() -> Self {
        Self {
            document_versions: Arc::new(RwLock::new(HashMap::new())),
            editor_decorations: Arc::new(RwLock::new(HashMap::new())),
            decoration_types: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Open a text document and return its metadata as a JSON value.
    pub async fn open_text_document(&self, path: &str) -> Result<Value, String> {
        let content = fs::read_to_string(path).unwrap_or_default();
        let eol = if content.contains("\r\n") { 2 } else { 1 };
        let version = self.get_document_version(path).await;

        Ok(json!({
            "uri": path,
            "text": content,
            "languageId": detect_language_id(path),
            "version": version,
            "isDirty": false,
            "eol": eol,
        }))
    }

    /// Persist document content to disk and bump the version.
    pub async fn persist_document(&self, path: &str, content: &str) -> Result<i32, String> {
        if let Some(parent) = Path::new(path).parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                return Err(format!(
                    "Failed to prepare document directory {:?}: {}",
                    parent, err
                ));
            }
        }

        fs::write(path, content).map_err(|e| format!("Failed to save document {}: {}", path, e))?;

        let version = self.bump_document_version(path).await;
        Ok(version)
    }

    /// Get the current document version.
    pub async fn get_document_version(&self, path: &str) -> i32 {
        let mut map = self.document_versions.write().await;
        let entry = map.entry(path.to_string()).or_insert(1);
        *entry
    }

    /// Bump and return the new document version.
    pub async fn bump_document_version(&self, path: &str) -> i32 {
        let mut map = self.document_versions.write().await;
        let entry = map.entry(path.to_string()).or_insert(1);
        *entry += 1;
        *entry
    }

    /// Convert a VS Code snippet string to plain text.
    pub fn snippet_to_plain(snippet: &str) -> String {
        let placeholder =
            Regex::new(r"\$\{(\d+):([^}]*)\}").expect("snippet placeholder regex is valid");
        let tabstop = Regex::new(r"\$(\d+)").expect("snippet tabstop regex is valid");
        let mut result = placeholder.replace_all(snippet, "$2").into_owned();
        result = tabstop.replace_all(&result, "").into_owned();
        result.replace("\\$", "$")
    }

    /// Parse snippet locations from a JSON value.
    pub fn parse_snippet_locations(location: Option<&Value>) -> Vec<RangePayload> {
        fn push_location(target: &mut Vec<RangePayload>, value: &Value) {
            if let Ok(range) = serde_json::from_value::<RangePayload>(value.clone()) {
                target.push(range);
                return;
            }

            if let Ok(position) = serde_json::from_value::<PositionPayload>(value.clone()) {
                target.push(RangePayload::from_position(position));
            }
        }

        let mut ranges = Vec::new();
        match location {
            Some(Value::Array(items)) => {
                for item in items {
                    push_location(&mut ranges, item);
                }
            }
            Some(value) => push_location(&mut ranges, value),
            None => {}
        }

        if ranges.is_empty() {
            ranges.push(RangePayload::from_position(PositionPayload {
                line: 0,
                character: 0,
            }));
        }

        ranges
    }

    /// Apply text edits to a document, returning the modified text.
    pub fn apply_text_edits(
        original: &str,
        edits: &mut [TextEditPayload],
        end_of_line: Option<i32>,
    ) -> Result<String, String> {
        let mut normalized = original.replace("\r\n", "\n");
        let original_crlf = original.contains("\r\n");

        edits.sort_by(|a, b| {
            b.range
                .start
                .line
                .cmp(&a.range.start.line)
                .then_with(|| b.range.start.character.cmp(&a.range.start.character))
        });

        for edit in edits.iter() {
            let line_offsets = build_line_offsets(&normalized);
            let start = offset_for_position(&normalized, &line_offsets, &edit.range.start)?;
            let end = offset_for_position(&normalized, &line_offsets, &edit.range.end)?;
            let replacement = edit.new_text.replace("\r\n", "\n");
            normalized.replace_range(start..end, &replacement);
        }

        let newline = match end_of_line {
            Some(2) => "\r\n",
            Some(1) => "\n",
            _ => {
                if original_crlf {
                    "\r\n"
                } else {
                    "\n"
                }
            }
        };

        let result = if newline == "\n" {
            normalized
        } else {
            normalized.replace("\n", newline)
        };

        Ok(result)
    }

    /// Update decorations for a given URI/key.
    pub async fn update_decorations(
        &self,
        uri: &str,
        key: &str,
        decorations: Value,
    ) -> Result<Vec<Value>, String> {
        let normalized = self.normalize_decorations(key, decorations).await?;

        {
            let mut map = self.editor_decorations.write().await;
            let entry = map.entry(uri.to_string()).or_insert_with(HashMap::new);
            if normalized.is_empty() {
                entry.remove(key);
                if entry.is_empty() {
                    map.remove(uri);
                }
            } else {
                entry.insert(key.to_string(), Value::Array(normalized.clone()));
            }
        }

        Ok(normalized)
    }

    /// Dispose decorations for a given key across all URIs.
    pub async fn dispose_decorations(&self, key: &str) -> Result<Vec<String>, String> {
        let mut affected = Vec::new();
        {
            let mut map = self.editor_decorations.write().await;
            for (uri, decorations) in map.iter_mut() {
                if decorations.remove(key).is_some() {
                    affected.push(uri.clone());
                }
            }
            map.retain(|_, decorations| !decorations.is_empty());
        }
        {
            let mut types = self.decoration_types.write().await;
            types.remove(key);
        }

        Ok(affected)
    }

    async fn normalize_decorations(
        &self,
        key: &str,
        decorations: Value,
    ) -> Result<Vec<Value>, String> {
        let base_options = {
            let map = self.decoration_types.read().await;
            map.get(key).cloned().unwrap_or(Value::Object(Map::new()))
        };

        let mut normalized = Vec::new();
        let entries = match decorations {
            Value::Array(arr) => arr,
            Value::Null => Vec::new(),
            other => vec![other],
        };

        for entry in entries {
            let (range, hover, specific_options) =
                if let Ok(range) = serde_json::from_value::<RangePayload>(entry.clone()) {
                    (range, None, None)
                } else if let Some(range_value) = entry.get("range") {
                    let range = serde_json::from_value::<RangePayload>(range_value.clone())
                        .map_err(|e| format!("Invalid decoration range payload: {}", e))?;
                    let hover = entry.get("hoverMessage").cloned();
                    let specific = entry
                        .get("renderOptions")
                        .or_else(|| entry.get("options"))
                        .cloned();
                    (range, hover, specific)
                } else {
                    continue;
                };

            let merged_options = merge_decoration_options(&base_options, specific_options.as_ref());
            let mut map = Map::new();
            map.insert(
                "range".into(),
                serde_json::to_value(&range).unwrap_or(Value::Null),
            );
            map.insert("options".into(), merged_options);
            if let Some(hover_msg) = hover {
                map.insert("hoverMessage".into(), hover_msg);
            }
            normalized.push(Value::Object(map));
        }

        Ok(normalized)
    }
}

fn merge_decoration_options(base: &Value, specific: Option<&Value>) -> Value {
    let mut merged = match base {
        Value::Object(obj) => obj.clone(),
        _ => Map::new(),
    };

    if let Some(Value::Object(spec)) = specific {
        for (key, value) in spec {
            merged.insert(key.clone(), value.clone());
        }
    }

    Value::Object(merged)
}

/// Detect language ID from file extension.
pub fn detect_language_id(path: &str) -> &'static str {
    match Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "rs" => "rust",
        "ts" | "tsx" => "typescript",
        "js" | "jsx" => "javascript",
        "py" => "python",
        "java" => "java",
        "cs" => "csharp",
        "cpp" | "cxx" | "cc" | "h" | "hpp" => "cpp",
        "go" => "go",
        "rb" => "ruby",
        "swift" => "swift",
        "kt" => "kotlin",
        "php" => "php",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "md" => "markdown",
        "html" | "htm" => "html",
        "css" | "scss" | "less" => "css",
        _ => "plaintext",
    }
}

fn build_line_offsets(text: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    for (idx, ch) in text.char_indices() {
        if ch == '\n' {
            offsets.push(idx + 1);
        }
    }
    offsets
}

fn offset_for_position(
    text: &str,
    offsets: &[usize],
    position: &PositionPayload,
) -> Result<usize, String> {
    if offsets.is_empty() {
        return Ok(0);
    }

    let line_index = position.line.min(offsets.len() - 1);
    let line_start = offsets[line_index];
    let line_end = if line_index + 1 < offsets.len() {
        offsets[line_index + 1].saturating_sub(1)
    } else {
        text.len()
    };

    let line_text = &text[line_start..line_end];
    let total_chars = line_text.chars().count();
    let target_chars = position.character.min(total_chars);

    let mut byte_offset = line_start;
    for (consumed, (idx, ch)) in line_text.char_indices().enumerate() {
        if consumed == target_chars {
            byte_offset = line_start + idx;
            break;
        }
        byte_offset = line_start + idx + ch.len_utf8();
    }

    if target_chars == total_chars {
        byte_offset = line_end;
    }

    Ok(byte_offset)
}

impl Default for DocumentManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── detect_language_id tests ─────────────────────────────────────────────────

    #[test]
    fn test_detect_language_id_rust() {
        assert_eq!(detect_language_id("main.rs"), "rust");
        assert_eq!(detect_language_id("/home/user/src/lib.rs"), "rust");
    }

    #[test]
    fn test_detect_language_id_typescript() {
        assert_eq!(detect_language_id("app.ts"), "typescript");
        assert_eq!(detect_language_id("component.tsx"), "typescript");
    }

    #[test]
    fn test_detect_language_id_javascript() {
        assert_eq!(detect_language_id("index.js"), "javascript");
        assert_eq!(detect_language_id("App.jsx"), "javascript");
    }

    #[test]
    fn test_detect_language_id_python() {
        assert_eq!(detect_language_id("script.py"), "python");
    }

    #[test]
    fn test_detect_language_id_go() {
        assert_eq!(detect_language_id("main.go"), "go");
    }

    #[test]
    fn test_detect_language_id_other_common() {
        assert_eq!(detect_language_id("App.java"), "java");
        assert_eq!(detect_language_id("main.cpp"), "cpp");
        assert_eq!(detect_language_id("app.rb"), "ruby");
        assert_eq!(detect_language_id("main.swift"), "swift");
        assert_eq!(detect_language_id("app.kt"), "kotlin");
        assert_eq!(detect_language_id("index.html"), "html");
        assert_eq!(detect_language_id("style.css"), "css");
        assert_eq!(detect_language_id("data.json"), "json");
        assert_eq!(detect_language_id("config.yaml"), "yaml");
        assert_eq!(detect_language_id("Cargo.toml"), "toml");
        assert_eq!(detect_language_id("README.md"), "markdown");
    }

    #[test]
    fn test_detect_language_id_unknown() {
        assert_eq!(detect_language_id("file.xyz"), "plaintext");
        assert_eq!(detect_language_id("noext"), "plaintext");
        assert_eq!(detect_language_id(""), "plaintext");
    }

    #[test]
    fn test_detect_language_id_case_insensitive() {
        assert_eq!(detect_language_id("main.RS"), "rust");
        assert_eq!(detect_language_id("app.TS"), "typescript");
        assert_eq!(detect_language_id("script.PY"), "python");
    }

    // ── DocumentManager tests ────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_document_manager_new() {
        let dm = DocumentManager::new();
        let version = dm.get_document_version("test.rs").await;
        assert_eq!(version, 1);
    }

    #[tokio::test]
    async fn test_document_version_bump() {
        let dm = DocumentManager::new();
        let v1 = dm.get_document_version("test.rs").await;
        assert_eq!(v1, 1);
        let v2 = dm.bump_document_version("test.rs").await;
        assert_eq!(v2, 2);
        let v3 = dm.bump_document_version("test.rs").await;
        assert_eq!(v3, 3);
    }

    #[test]
    fn test_snippet_to_plain() {
        assert_eq!(
            DocumentManager::snippet_to_plain("hello ${1:world}"),
            "hello world"
        );
        assert_eq!(DocumentManager::snippet_to_plain("foo $1 bar"), "foo  bar");
        assert_eq!(
            DocumentManager::snippet_to_plain("dollar \\$ sign"),
            "dollar $ sign"
        );
    }

    #[test]
    fn test_parse_snippet_locations_empty() {
        let ranges = DocumentManager::parse_snippet_locations(None);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start.line, 0);
        assert_eq!(ranges[0].start.character, 0);
    }

    #[test]
    fn test_parse_snippet_locations_with_position() {
        let pos = serde_json::json!({ "line": 5, "character": 10 });
        let ranges = DocumentManager::parse_snippet_locations(Some(&pos));
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start.line, 5);
        assert_eq!(ranges[0].start.character, 10);
    }

    #[test]
    fn test_position_payload_serde_roundtrip() {
        let pos = PositionPayload {
            line: 3,
            character: 15,
        };
        let json = serde_json::to_string(&pos).unwrap();
        let deserialized: PositionPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.line, pos.line);
        assert_eq!(deserialized.character, pos.character);
    }

    #[test]
    fn test_range_payload_serde_roundtrip() {
        let range = RangePayload {
            start: PositionPayload {
                line: 0,
                character: 5,
            },
            end: PositionPayload {
                line: 0,
                character: 10,
            },
        };
        let json = serde_json::to_string(&range).unwrap();
        let deserialized: RangePayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.start.line, range.start.line);
        assert_eq!(deserialized.start.character, range.start.character);
        assert_eq!(deserialized.end.line, range.end.line);
        assert_eq!(deserialized.end.character, range.end.character);
    }

    #[test]
    fn test_range_payload_from_position() {
        let pos = PositionPayload {
            line: 2,
            character: 8,
        };
        let range = RangePayload::from_position(pos);
        assert_eq!(range.start.line, range.end.line);
        assert_eq!(range.start.character, range.end.character);
        assert_eq!(range.start.line, 2);
        assert_eq!(range.start.character, 8);
    }

    #[test]
    fn test_apply_text_edits_simple() {
        let original = "hello world";
        let mut edits = vec![TextEditPayload {
            range: RangePayload {
                start: PositionPayload {
                    line: 0,
                    character: 0,
                },
                end: PositionPayload {
                    line: 0,
                    character: 5,
                },
            },
            new_text: "goodbye".to_string(),
        }];
        let result = DocumentManager::apply_text_edits(original, &mut edits, Some(1)).unwrap();
        assert_eq!(result, "goodbye world");
    }

    #[test]
    fn test_apply_text_edits_multiple() {
        let original = "abc def ghi";
        let mut edits = vec![
            TextEditPayload {
                range: RangePayload {
                    start: PositionPayload {
                        line: 0,
                        character: 4,
                    },
                    end: PositionPayload {
                        line: 0,
                        character: 7,
                    },
                },
                new_text: "XYZ".to_string(),
            },
            TextEditPayload {
                range: RangePayload {
                    start: PositionPayload {
                        line: 0,
                        character: 0,
                    },
                    end: PositionPayload {
                        line: 0,
                        character: 3,
                    },
                },
                new_text: "123".to_string(),
            },
        ];
        let result = DocumentManager::apply_text_edits(original, &mut edits, Some(1)).unwrap();
        assert_eq!(result, "123 XYZ ghi");
    }
}
