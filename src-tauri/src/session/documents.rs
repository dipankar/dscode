use super::{PositionPayload, RangePayload, SessionEvent, SessionManager, TextEditPayload};
use crate::commands::TextDocumentContentChange;
use regex::Regex;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

impl SessionManager {
    pub(super) async fn open_text_document(&self, path: &str) -> Result<Value, String> {
        let content = fs::read_to_string(path).unwrap_or_default();
        let eol = if content.contains("\r\n") { 2 } else { 1 };
        let version = self.get_document_version(path).await;

        Ok(json!({
            "uri": path,
            "text": content,
            "languageId": Self::detect_language_id(path),
            "version": version,
            "isDirty": false,
            "eol": eol,
        }))
    }

    pub(super) async fn persist_document(&self, path: &str, content: &str) -> Result<i32, String> {
        if let Some(parent) = Path::new(path).parent() {
            if let Err(err) = fs::create_dir_all(parent) {
                return Err(format!("Failed to prepare document directory {:?}: {}", parent, err));
            }
        }

        fs::write(path, content).map_err(|e| format!("Failed to save document {}: {}", path, e))?;

        let version = self.bump_document_version(path).await;
        self.emit_event(SessionEvent::DocumentChanged {
            path: path.to_string(),
            content: content.to_string(),
        });

        Ok(version)
    }

    async fn get_document_version(&self, path: &str) -> i32 {
        let mut map = self.document_versions.write().await;
        let entry = map.entry(path.to_string()).or_insert(1);
        *entry
    }

    async fn bump_document_version(&self, path: &str) -> i32 {
        let mut map = self.document_versions.write().await;
        let entry = map.entry(path.to_string()).or_insert(1);
        *entry += 1;
        *entry
    }

    fn detect_language_id(path: &str) -> &'static str {
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

    pub(super) fn snippet_to_plain(snippet: &str) -> String {
        let placeholder =
            Regex::new(r"\$\{(\d+):([^}]*)\}").expect("snippet placeholder regex is valid");
        let tabstop = Regex::new(r"\$(\d+)").expect("snippet tabstop regex is valid");
        let mut result = placeholder.replace_all(snippet, "$2").into_owned();
        result = tabstop.replace_all(&result, "").into_owned();
        result.replace("\\$", "$")
    }

    pub(super) fn parse_snippet_locations(location: Option<&Value>) -> Vec<RangePayload> {
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
            ranges.push(RangePayload::from_position(PositionPayload { line: 0, character: 0 }));
        }

        ranges
    }

    pub(super) fn apply_text_edits(
        original: &str, edits: &mut [TextEditPayload], end_of_line: Option<i32>,
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
            let line_offsets = Self::build_line_offsets(&normalized);
            let start = Self::offset_for_position(&normalized, &line_offsets, &edit.range.start)?;
            let end = Self::offset_for_position(&normalized, &line_offsets, &edit.range.end)?;
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

        let result = if newline == "\n" { normalized } else { normalized.replace("\n", newline) };

        Ok(result)
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
        text: &str, offsets: &[usize], position: &PositionPayload,
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

    pub(super) async fn update_decorations(
        &self, uri: &str, key: &str, decorations: Value,
    ) -> Result<(), String> {
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

        self.emit_event(SessionEvent::EditorDecorations {
            uri: uri.to_string(),
            key: key.to_string(),
            decorations: Value::Array(normalized),
        });

        Ok(())
    }

    pub(super) async fn dispose_decorations(&self, key: &str) -> Result<(), String> {
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

        for uri in affected {
            self.emit_event(SessionEvent::EditorDecorations {
                uri: uri.clone(),
                key: key.to_string(),
                decorations: Value::Array(Vec::new()),
            });
        }

        Ok(())
    }

    async fn normalize_decorations(
        &self, key: &str, decorations: Value,
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
            let (range, hover, specific_options) = if let Ok(range) =
                serde_json::from_value::<RangePayload>(entry.clone())
            {
                (range, None, None)
            } else if let Some(range_value) = entry.get("range") {
                let range = serde_json::from_value::<RangePayload>(range_value.clone())
                    .map_err(|e| format!("Invalid decoration range payload: {}", e))?;
                let hover = entry.get("hoverMessage").cloned();
                let specific = entry.get("renderOptions").or_else(|| entry.get("options")).cloned();
                (range, hover, specific)
            } else {
                continue;
            };

            let merged_options =
                Self::merge_decoration_options(&base_options, specific_options.as_ref());
            let mut map = Map::new();
            map.insert("range".into(), serde_json::to_value(&range).unwrap_or(Value::Null));
            map.insert("options".into(), merged_options);
            if let Some(hover_msg) = hover {
                map.insert("hoverMessage".into(), hover_msg);
            }
            normalized.push(Value::Object(map));
        }

        Ok(normalized)
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

    pub async fn notify_editor_selection(
        &self, uri: &str, selection: Value, selections: Value,
    ) -> Result<(), String> {
        let payload = json!({
            "uri": uri,
            "selection": selection,
            "selections": selections,
        });
        self.ipc_manager.request("main", "selectionChanged", payload).await?;
        Ok(())
    }

    pub async fn notify_editor_visible_ranges(
        &self, uri: &str, ranges: Value,
    ) -> Result<(), String> {
        let payload = json!({
            "uri": uri,
            "ranges": ranges,
        });
        self.ipc_manager.request("main", "visibleRangesChanged", payload).await?;
        Ok(())
    }

    pub async fn forward_document_change(
        &self, uri: &str, version: u64, changes: &[TextDocumentContentChange],
    ) -> Result<(), String> {
        let content_changes: Vec<Value> = changes
            .iter()
            .map(|change| {
                let range = change.range.as_ref().map(|r| {
                    json!({
                        "startLineNumber": r.start.line + 1,
                        "startColumn": r.start.character + 1,
                        "endLineNumber": r.end.line + 1,
                        "endColumn": r.end.character + 1,
                    })
                });
                json!({
                    "range": range,
                    "rangeOffset": change.range_offset,
                    "rangeLength": change.range_length,
                    "text": change.text,
                })
            })
            .collect();

        let payload = json!({
            "uri": uri,
            "version": version,
            "contentChanges": content_changes,
        });

        let _ = self.ipc_manager.request("main", "textDocumentChanged", payload).await;

        Ok(())
    }
}
