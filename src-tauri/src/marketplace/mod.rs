/**
 * VS Code Marketplace Integration
 *
 * Provides access to the official VS Code Extension Marketplace
 * API endpoint: https://marketplace.visualstudio.com/_apis/public/gallery
 */
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

static HTTP_CLIENT: once_cell::sync::Lazy<reqwest::Client> =
    once_cell::sync::Lazy::new(reqwest::Client::new);

const MARKETPLACE_API_URL: &str =
    "https://marketplace.visualstudio.com/_apis/public/gallery/extensionquery";
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
    query: String, page: i32, page_size: i32,
) -> Result<Vec<MarketplaceExtension>, String> {
    let marketplace_query = MarketplaceQuery {
        filters: vec![Filter {
            criteria: vec![Criterion {
                filter_type: 10, // SearchText
                value: query,
            }],
            page_number: page,
            page_size,
            sort_by: 4,    // InstallCount
            sort_order: 2, // Descending
        }],
        flags: 0x192, // IncludeVersions | IncludeFiles | IncludeStatistics
    };

    let response = HTTP_CLIENT
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
    publisher: String, extension_name: String,
) -> Result<MarketplaceExtension, String> {
    let marketplace_query = MarketplaceQuery {
        filters: vec![Filter {
            criteria: vec![Criterion {
                filter_type: 7, // ExtensionName
                value: format!("{}.{}", publisher, extension_name),
            }],
            page_number: 1,
            page_size: 1,
            sort_by: 0,
            sort_order: 0,
        }],
        flags: 0x192, // IncludeVersions | IncludeFiles | IncludeStatistics
    };

    let response = HTTP_CLIENT
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
    let result = marketplace_response.results.first().ok_or("No results found")?;

    let ext_raw = result.extensions.first().ok_or("Extension not found")?;

    let version = ext_raw.versions.first().ok_or("No versions found")?;

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
    publisher: String, extension_name: String, version: String,
) -> Result<PathBuf, String> {
    let marketplace_query = MarketplaceQuery {
        filters: vec![Filter {
            criteria: vec![Criterion {
                filter_type: 7, // ExtensionName
                value: format!("{}.{}", publisher, extension_name),
            }],
            page_number: 1,
            page_size: 1,
            sort_by: 0,
            sort_order: 0,
        }],
        flags: 0x192, // IncludeVersions | IncludeFiles | IncludeStatistics
    };

    let details_response = HTTP_CLIENT
        .post(MARKETPLACE_API_URL)
        .header("Accept", format!("application/json;api-version={}", MARKETPLACE_API_VERSION))
        .header("Content-Type", "application/json")
        .json(&marketplace_query)
        .send()
        .await
        .map_err(|e| format!("Failed to query marketplace for download URL: {}", e))?;

    let marketplace_response: MarketplaceResponse = details_response
        .json()
        .await
        .map_err(|e| format!("Failed to parse marketplace response: {}", e))?;

    // Get the VSIX download URL from the response
    let download_url = marketplace_response
        .results
        .first()
        .and_then(|r| r.extensions.first())
        .and_then(|ext| ext.versions.first())
        .and_then(|v| {
            v.files
                .iter()
                .find(|f| f.asset_type == "Microsoft.VisualStudio.Services.VSIXPackage")
                .map(|f| f.source.clone())
        })
        .ok_or("Could not find VSIX download URL in marketplace response")?;

    info!("[Marketplace] Downloading from: {}", download_url);

    // Download the .vsix file with proper headers
    let response = HTTP_CLIENT
        .get(&download_url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) VSCode/1.80.0")
        .header("Accept", "*/*")
        .send()
        .await
        .map_err(|e| format!("Failed to download extension: {}", e))?;

    debug!("[Marketplace] Response status: {}", response.status());

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    // Validate Content-Type (should be application/octet-stream or application/zip)
    if let Some(content_type) = response.headers().get("content-type") {
        let ct = content_type.to_str().unwrap_or("");
        if !ct.contains("application/octet-stream")
            && !ct.contains("application/zip")
            && !ct.contains("binary/octet-stream")
        {
            warn!("[Marketplace] Warning: Unexpected content-type: {}", ct);
        }
    }

    // Save to temporary file
    let temp_dir =
        tempfile::tempdir().map_err(|e| format!("Failed to create temp directory: {}", e))?;

    let vsix_path =
        temp_dir.path().join(format!("{}.{}-{}.vsix", publisher, extension_name, version));

    let bytes =
        response.bytes().await.map_err(|e| format!("Failed to read response bytes: {}", e))?;

    info!("[Marketplace] Downloaded {} bytes", bytes.len());

    if bytes.len() < 1024 {
        return Err(format!(
            "Downloaded file is too small ({} bytes), likely invalid",
            bytes.len()
        ));
    }

    if bytes.len() >= 4 && (&bytes[0..2] != b"PK") {
        let preview = if bytes.len() >= 200 {
            String::from_utf8_lossy(&bytes[0..200])
        } else {
            String::from_utf8_lossy(&bytes)
        };
        error!(
            "[Marketplace] ERROR: File is not a ZIP. First bytes: {:?}",
            &bytes[0..std::cmp::min(16, bytes.len())]
        );
        error!("[Marketplace] Preview: {}", preview);
        return Err("Downloaded file is not a valid ZIP archive (invalid magic number). This usually means the marketplace API returned an error page instead of the extension file.".to_string());
    }

    let vsix_path_clone = vsix_path.clone();
    tokio::task::spawn_blocking(move || {
        std::fs::write(&vsix_path_clone, &bytes)
            .map_err(|e| format!("Failed to write .vsix file: {}", e))?;

        let f = std::fs::File::open(&vsix_path_clone)
            .map_err(|e| format!("Failed to open downloaded file for validation: {}", e))?;
        zip::ZipArchive::new(f)
            .map_err(|e| format!("Downloaded file is not a valid VSIX archive: {}", e))?;

        Ok::<_, String>(())
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))??;

    let _ = temp_dir.keep();

    Ok(vsix_path)
}
