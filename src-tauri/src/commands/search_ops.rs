use dscode_core::CoreError;
use grep_matcher::Matcher;
use grep_regex::{RegexMatcher, RegexMatcherBuilder};
use grep_searcher::sinks::UTF8;
use grep_searcher::Searcher;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub line: usize,
    pub column: usize,
    pub text: String,
    pub r#match: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchOptions {
    pub query: String,
    pub case_sensitive: bool,
    pub use_regex: bool,
    pub whole_word: bool,
    pub include_pattern: Option<String>,
    pub exclude_pattern: Option<String>,
    pub max_results: Option<usize>,
}

#[tauri::command]
pub async fn search_in_files(
    root_path: String, options: SearchOptions,
) -> Result<Vec<SearchResult>, String> {
    // Move the blocking search operation to a background thread
    tokio::task::spawn_blocking(move || {
        perform_search(root_path, options).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Search task failed: {}", e))?
}

fn perform_search(
    root_path: String, options: SearchOptions,
) -> Result<Vec<SearchResult>, CoreError> {
    let root = PathBuf::from(&root_path);

    if !root.exists() {
        return Err(CoreError::PathResolution(format!("Path does not exist: {}", root_path)));
    }

    let mut results = Vec::new();
    let max_results = options.max_results.unwrap_or(1000);

    // Build regex matcher
    let pattern = if options.whole_word {
        format!(r"\b{}\b", regex::escape(&options.query))
    } else if !options.use_regex {
        regex::escape(&options.query)
    } else {
        options.query.clone()
    };

    let matcher = RegexMatcherBuilder::new()
        .case_insensitive(!options.case_sensitive)
        .build(&pattern)
        .map_err(|e| CoreError::Config(format!("Invalid regex: {}", e)))?;

    // Build file walker with limits
    let mut walker_builder = WalkBuilder::new(&root);
    walker_builder
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .threads(4) // Use parallel threads for faster searching
        .max_depth(Some(20)); // Limit depth to prevent infinite recursion

    // Add include/exclude patterns
    if let Some(include) = &options.include_pattern {
        if !include.is_empty() {
            walker_builder.types(
                ignore::types::TypesBuilder::new()
                    .add_defaults()
                    .select("all")
                    .build()
                    .map_err(|e| CoreError::Config(format!("Invalid include pattern: {}", e)))?,
            );
        }
    }

    let walker = walker_builder.build();

    // Search through files
    for entry in walker {
        if results.len() >= max_results {
            break;
        }

        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();

        // Skip directories
        if !path.is_file() {
            continue;
        }

        // Check exclude pattern
        if let Some(exclude) = &options.exclude_pattern {
            if !exclude.is_empty() {
                if let Some(path_str) = path.to_str() {
                    if path_str.contains(exclude) {
                        continue;
                    }
                }
            }
        }

        // Search in file
        if let Ok(file_results) = search_file(&matcher, path, &root, max_results - results.len()) {
            results.extend(file_results);
        }
    }

    Ok(results)
}

fn search_file(
    matcher: &RegexMatcher, path: &Path, root: &Path, max_matches: usize,
) -> Result<Vec<SearchResult>, CoreError> {
    let mut results = Vec::new();
    let mut searcher = Searcher::new();

    // Get relative path
    let relative_path = path.strip_prefix(root).unwrap_or(path).to_string_lossy().to_string();

    searcher
        .search_path(
            matcher,
            path,
            UTF8(|lnum, line| {
                if results.len() >= max_matches {
                    return Ok(false); // Stop searching
                }

                let line_str = line.trim_end_matches('\n');

                // Find first match in the line
                let mut found_match = false;
                let _ = matcher.find_iter(line_str.as_bytes(), |m| {
                    if !found_match {
                        let match_start = m.start();
                        let match_end = m.end();
                        let match_text = &line_str[match_start..match_end];

                        results.push(SearchResult {
                            path: relative_path.clone(),
                            line: lnum as usize,
                            column: match_start + 1,
                            text: line_str.to_string(),
                            r#match: match_text.to_string(),
                        });

                        found_match = true;
                        false // Stop after first match
                    } else {
                        false
                    }
                });

                if !found_match {
                    return Ok(true); // Continue to next line if no match
                }

                Ok(true) // Continue searching
            }),
        )
        .map_err(|e| CoreError::Io(std::io::Error::other(format!("Search error: {}", e))))?;

    Ok(results)
}

#[tauri::command]
pub async fn replace_in_files(
    root_path: String, _search_query: String, replace_text: String, options: SearchOptions,
) -> Result<usize, String> {
    let root = PathBuf::from(&root_path);

    if !root.exists() {
        return Err(
            CoreError::PathResolution(format!("Path does not exist: {}", root_path)).to_string()
        );
    }

    // First, find all matches
    let search_results = search_in_files(root_path.clone(), options.clone()).await?;

    // Group by file
    let mut files_modified = 0;
    let mut file_map: std::collections::HashMap<String, Vec<SearchResult>> =
        std::collections::HashMap::new();

    for result in search_results {
        file_map.entry(result.path.clone()).or_default().push(result);
    }

    // Replace in each file
    for (file_path, matches) in file_map {
        let full_path = root.join(&file_path);

        if let Ok(content) = std::fs::read_to_string(&full_path) {
            let mut new_content = content.clone();

            // Replace from end to start to maintain positions
            let mut sorted_matches = matches;
            sorted_matches.sort_by(|a, b| b.line.cmp(&a.line).then(b.column.cmp(&a.column)));

            for m in sorted_matches {
                new_content = new_content.replace(&m.r#match, &replace_text);
            }

            if new_content != content {
                std::fs::write(&full_path, new_content)
                    .map_err(|e| CoreError::from(e).to_string())?;
                files_modified += 1;
            }
        }
    }

    Ok(files_modified)
}
