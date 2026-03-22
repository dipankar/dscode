use super::marketplace_ui_registry::*;
use tauri::State;

// ===== Featured Extensions =====

/// Set featured extensions
#[tauri::command]
pub async fn set_featured_extensions(
    extension_ids: Vec<String>,
    registry: State<'_, MarketplaceUIRegistry>,
) -> Result<(), String> {
    registry.set_featured_extensions(extension_ids).await;
    Ok(())
}

/// Get featured extensions
#[tauri::command]
pub async fn get_featured_extensions(
    registry: State<'_, MarketplaceUIRegistry>,
) -> Result<Vec<String>, String> {
    Ok(registry.get_featured_extensions().await)
}

// ===== Categories =====

/// Get all extension categories
#[tauri::command]
pub async fn get_extension_categories(
    registry: State<'_, MarketplaceUIRegistry>,
) -> Result<Vec<ExtensionCategory>, String> {
    Ok(registry.get_categories().await)
}

/// Get extension category by ID
#[tauri::command]
pub async fn get_extension_category(
    category_id: String,
    registry: State<'_, MarketplaceUIRegistry>,
) -> Result<Option<ExtensionCategory>, String> {
    Ok(registry.get_category(&category_id).await)
}

// ===== Reviews =====

/// Add extension review
#[tauri::command]
pub async fn add_extension_review(
    extension_id: String,
    review: ExtensionReview,
    registry: State<'_, MarketplaceUIRegistry>,
) -> Result<(), String> {
    registry.add_review(extension_id, review).await
}

/// Get extension reviews
#[tauri::command]
pub async fn get_extension_reviews(
    extension_id: String,
    registry: State<'_, MarketplaceUIRegistry>,
) -> Result<Vec<ExtensionReview>, String> {
    Ok(registry.get_reviews(&extension_id).await)
}

/// Get extension reviews with pagination
#[tauri::command]
pub async fn get_extension_reviews_paginated(
    extension_id: String,
    page: usize,
    page_size: usize,
    registry: State<'_, MarketplaceUIRegistry>,
) -> Result<ReviewsPage, String> {
    Ok(registry.get_reviews_paginated(&extension_id, page, page_size).await)
}

// ===== Ratings =====

/// Get extension rating
#[tauri::command]
pub async fn get_extension_rating(
    extension_id: String,
    registry: State<'_, MarketplaceUIRegistry>,
) -> Result<Option<ExtensionRating>, String> {
    Ok(registry.get_rating(&extension_id).await)
}

// ===== Recommendations =====

/// Add extension recommendations
#[tauri::command]
pub async fn add_extension_recommendations(
    extension_id: String,
    recommended_ids: Vec<String>,
    registry: State<'_, MarketplaceUIRegistry>,
) -> Result<(), String> {
    registry.add_recommendation(extension_id, recommended_ids).await;
    Ok(())
}

/// Get extension recommendations
#[tauri::command]
pub async fn get_extension_recommendations(
    extension_id: String,
    registry: State<'_, MarketplaceUIRegistry>,
) -> Result<Vec<String>, String> {
    Ok(registry.get_recommendations(&extension_id).await)
}

/// Generate recommendations based on installed extensions
#[tauri::command]
pub async fn generate_recommendations(
    installed_extensions: Vec<String>,
    registry: State<'_, MarketplaceUIRegistry>,
) -> Result<Vec<String>, String> {
    Ok(registry.generate_recommendations(installed_extensions).await)
}

// ===== Update Notifications =====

/// Add update notification
#[tauri::command]
pub async fn add_update_notification(
    notification: UpdateNotification,
    registry: State<'_, MarketplaceUIRegistry>,
) -> Result<(), String> {
    registry.add_update_notification(notification).await;
    Ok(())
}

/// Get all update notifications
#[tauri::command]
pub async fn get_update_notifications(
    registry: State<'_, MarketplaceUIRegistry>,
) -> Result<Vec<UpdateNotification>, String> {
    Ok(registry.get_update_notifications().await)
}

/// Remove update notification
#[tauri::command]
pub async fn remove_update_notification(
    extension_id: String,
    registry: State<'_, MarketplaceUIRegistry>,
) -> Result<(), String> {
    registry.remove_update_notification(&extension_id).await;
    Ok(())
}

/// Clear all update notifications
#[tauri::command]
pub async fn clear_update_notifications(
    registry: State<'_, MarketplaceUIRegistry>,
) -> Result<(), String> {
    registry.clear_update_notifications().await;
    Ok(())
}

// ===== Search & Filter =====

/// Search extensions with filters
#[tauri::command]
pub async fn search_marketplace_extensions(
    query: String,
    category: Option<String>,
    sort_by: SortBy,
    registry: State<'_, MarketplaceUIRegistry>,
) -> Result<Vec<ExtensionSearchResult>, String> {
    Ok(registry.search_extensions(&query, category, sort_by).await)
}

// ===== Cleanup =====

/// Clear all marketplace data
#[tauri::command]
pub async fn clear_marketplace_data(
    registry: State<'_, MarketplaceUIRegistry>,
) -> Result<(), String> {
    registry.clear_all_data().await;
    Ok(())
}
