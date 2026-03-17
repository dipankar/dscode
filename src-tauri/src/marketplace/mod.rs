/**
 * VS Code Marketplace Integration
 *
 * Provides access to the official VS Code Extension Marketplace
 * API endpoint: https://marketplace.visualstudio.com/_apis/public/gallery
 */

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const MARKETPLACE_API_URL: &str = "https://marketplace.visualstudio.com/_apis/public/gallery/extensionquery";
const MARKETPLACE_API_VERSION: &str = "3.0-preview.1";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketplaceExtension {
    pub publisher: String,
    pub name: String,
    pub display_name: String,
    pub version: String,
    pub description: Option<String>,
    pub install_count: i64,
    pub rating: f64,
    pub rating_count: i64,
    pub icon_url: Option<String>,
    pub repository_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct MarketplaceQuery {
    filters: Vec<Filter>,
    flags: i32,
}

#[derive(Debug, Serialize)]
struct Filter {
    criteria: Vec<Criterion>,
    #[serde(rename = "pageNumber")]
    page_number: i32,
    #[serde(rename = "pageSize")]
    page_size: i32,
    #[serde(rename = "sortBy")]
    sort_by: i32,
    #[serde(rename = "sortOrder")]
    sort_order: i32,
}

#[derive(Debug, Serialize)]
struct Criterion {
    #[serde(rename = "filterType")]
    filter_type: i32,
    value: String,
}

#[derive(Debug, Deserialize)]
struct MarketplaceResponse {
    results: Vec<MarketplaceResult>,
}

#[derive(Debug, Deserialize)]
struct MarketplaceResult {
    extensions: Vec<MarketplaceExtensionRaw>,
}

#[derive(Debug, Deserialize)]
struct MarketplaceExtensionRaw {
    publisher: MarketplacePublisher,
    #[serde(rename = "extensionName")]
    extension_name: String,
    #[serde(rename = "displayName")]
    display_name: String,
    #[serde(rename = "shortDescription")]
    short_description: Option<String>,
    versions: Vec<MarketplaceVersion>,
    statistics: Vec<MarketplaceStatistic>,
}

#[derive(Debug, Deserialize)]
struct MarketplacePublisher {
    #[serde(rename = "publisherName")]
    publisher_name: String,
}

#[derive(Debug, Deserialize)]
struct MarketplaceVersion {
    version: String,
    files: Vec<MarketplaceFile>,
}

#[derive(Debug, Deserialize)]
struct MarketplaceFile {
    #[serde(rename = "assetType")]
    asset_type: String,
    source: String,
}

#[derive(Debug, Deserialize)]
struct MarketplaceStatistic {
    #[serde(rename = "statisticName")]
    statistic_name: String,
    value: f64,
}

/// Search the VS Code Marketplace
pub async fn search_marketplace(
    query: String,
    page: i32,
    page_size: i32,
) -> Result<Vec<MarketplaceExtension>, String> {
    let client = reqwest::Client::new();

    // Build marketplace query
    let marketplace_query = MarketplaceQuery {
        filters: vec![Filter {
            criteria: vec![Criterion {
                filter_type: 10, // SearchText
                value: query,
            }],
            page_number: page,
            page_size,
            sort_by: 4, // InstallCount
            sort_order: 2, // Descending
        }],
        flags: 0x192, // IncludeVersions | IncludeFiles | IncludeStatistics
    };

    // Make request to marketplace API
    let response = client
        .post(MARKETPLACE_API_URL)
        .header("Accept", format!("application/json;api-version={}", MARKETPLACE_API_VERSION))
        .header("Content-Type", "application/json")
        .json(&marketplace_query)
        .send()
        .await
        .map_err(|e| format!("Failed to query marketplace: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Marketplace API error: {}", response.status()));
    }

    let marketplace_response: MarketplaceResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse marketplace response: {}", e))?;

    // Parse and convert extensions
    let mut extensions = Vec::new();

    if let Some(result) = marketplace_response.results.first() {
        for ext_raw in &result.extensions {
            if let Some(version) = ext_raw.versions.first() {
                // Get icon URL
                let icon_url = version
                    .files
                    .iter()
                    .find(|f| f.asset_type == "Microsoft.VisualStudio.Services.Icons.Default")
                    .map(|f| f.source.clone());

                // Get repository URL
                let repository_url = version
                    .files
                    .iter()
                    .find(|f| f.asset_type == "Microsoft.VisualStudio.Code.Manifest")
                    .map(|f| f.source.clone());

                // Get statistics
                let install_count = ext_raw
                    .statistics
                    .iter()
                    .find(|s| s.statistic_name == "install")
                    .map(|s| s.value as i64)
                    .unwrap_or(0);

                let rating = ext_raw
                    .statistics
                    .iter()
                    .find(|s| s.statistic_name == "averagerating")
                    .map(|s| s.value)
                    .unwrap_or(0.0);

                let rating_count = ext_raw
                    .statistics
                    .iter()
                    .find(|s| s.statistic_name == "ratingcount")
                    .map(|s| s.value as i64)
                    .unwrap_or(0);

                extensions.push(MarketplaceExtension {
                    publisher: ext_raw.publisher.publisher_name.clone(),
                    name: ext_raw.extension_name.clone(),
                    display_name: ext_raw.display_name.clone(),
                    version: version.version.clone(),
                    description: ext_raw.short_description.clone(),
                    install_count,
                    rating,
                    rating_count,
                    icon_url,
                    repository_url,
                });
            }
        }
    }

    Ok(extensions)
}

/// Get detailed information about a specific extension
pub async fn get_extension_details(
    publisher: String,
    extension_name: String,
) -> Result<MarketplaceExtension, String> {
    let client = reqwest::Client::new();

    // Build marketplace query for specific extension
    let marketplace_query = MarketplaceQuery {
        filters: vec![Filter {
            criteria: vec![
                Criterion {
                    filter_type: 7, // ExtensionName
                    value: format!("{}.{}", publisher, extension_name),
                },
            ],
            page_number: 1,
            page_size: 1,
            sort_by: 0,
            sort_order: 0,
        }],
        flags: 0x192, // IncludeVersions | IncludeFiles | IncludeStatistics
    };

    // Make request
    let response = client
        .post(MARKETPLACE_API_URL)
        .header("Accept", format!("application/json;api-version={}", MARKETPLACE_API_VERSION))
        .header("Content-Type", "application/json")
        .json(&marketplace_query)
        .send()
        .await
        .map_err(|e| format!("Failed to query marketplace: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Marketplace API error: {}", response.status()));
    }

    let marketplace_response: MarketplaceResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse marketplace response: {}", e))?;

    // Parse extension
    let result = marketplace_response
        .results
        .first()
        .ok_or("No results found")?;

    let ext_raw = result
        .extensions
        .first()
        .ok_or("Extension not found")?;

    let version = ext_raw
        .versions
        .first()
        .ok_or("No versions found")?;

    // Get icon URL
    let icon_url = version
        .files
        .iter()
        .find(|f| f.asset_type == "Microsoft.VisualStudio.Services.Icons.Default")
        .map(|f| f.source.clone());

    // Get repository URL
    let repository_url = version
        .files
        .iter()
        .find(|f| f.asset_type == "Microsoft.VisualStudio.Code.Manifest")
        .map(|f| f.source.clone());

    // Get statistics
    let install_count = ext_raw
        .statistics
        .iter()
        .find(|s| s.statistic_name == "install")
        .map(|s| s.value as i64)
        .unwrap_or(0);

    let rating = ext_raw
        .statistics
        .iter()
        .find(|s| s.statistic_name == "averagerating")
        .map(|s| s.value)
        .unwrap_or(0.0);

    let rating_count = ext_raw
        .statistics
        .iter()
        .find(|s| s.statistic_name == "ratingcount")
        .map(|s| s.value as i64)
        .unwrap_or(0);

    Ok(MarketplaceExtension {
        publisher: ext_raw.publisher.publisher_name.clone(),
        name: ext_raw.extension_name.clone(),
        display_name: ext_raw.display_name.clone(),
        version: version.version.clone(),
        description: ext_raw.short_description.clone(),
        install_count,
        rating,
        rating_count,
        icon_url,
        repository_url,
    })
}

/// Download extension from marketplace
pub async fn download_extension(
    publisher: String,
    extension_name: String,
    version: String,
) -> Result<PathBuf, String> {
    let client = reqwest::Client::new();

    // Construct download URL using the marketplace API
    let download_url = format!(
        "https://marketplace.visualstudio.com/_apis/public/gallery/publishers/{}/vsextensions/{}/{}/vspackage",
        publisher, extension_name, version
    );

    // Download the .vsix file
    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download extension: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    // Save to temporary file
    let temp_dir = tempfile::tempdir()
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;

    let vsix_path = temp_dir.path().join(format!("{}.{}-{}.vsix", publisher, extension_name, version));

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response bytes: {}", e))?;

    std::fs::write(&vsix_path, bytes)
        .map_err(|e| format!("Failed to write .vsix file: {}", e))?;

    // Persist the temp directory by keeping it
    // This prevents it from being deleted when temp_dir goes out of scope
    let _ = temp_dir.keep();

    Ok(vsix_path)
}
