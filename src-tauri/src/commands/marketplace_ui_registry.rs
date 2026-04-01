use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use tracing::{error, info};

/// Marketplace UI registry for enhanced marketplace experience
pub struct MarketplaceUIRegistry {
    featured_extensions: Arc<RwLock<Vec<String>>>,
    categories: Arc<RwLock<Vec<ExtensionCategory>>>,
    reviews: Arc<RwLock<HashMap<String, Vec<ExtensionReview>>>>,
    ratings: Arc<RwLock<HashMap<String, ExtensionRating>>>,
    recommendations: Arc<RwLock<HashMap<String, Vec<String>>>>,
    update_notifications: Arc<RwLock<Vec<UpdateNotification>>>,
    app_handle: AppHandle,
}

impl MarketplaceUIRegistry {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            featured_extensions: Arc::new(RwLock::new(Vec::new())),
            categories: Arc::new(RwLock::new(Self::default_categories())),
            reviews: Arc::new(RwLock::new(HashMap::new())),
            ratings: Arc::new(RwLock::new(HashMap::new())),
            recommendations: Arc::new(RwLock::new(HashMap::new())),
            update_notifications: Arc::new(RwLock::new(Vec::new())),
            app_handle,
        }
    }

    fn default_categories() -> Vec<ExtensionCategory> {
        vec![
            ExtensionCategory {
                id: "programming-languages".to_string(),
                name: "Programming Languages".to_string(),
                description: Some("Language support and syntax highlighting".to_string()),
                icon: Some("code".to_string()),
            },
            ExtensionCategory {
                id: "themes".to_string(),
                name: "Themes".to_string(),
                description: Some("Color themes and icon themes".to_string()),
                icon: Some("palette".to_string()),
            },
            ExtensionCategory {
                id: "debuggers".to_string(),
                name: "Debuggers".to_string(),
                description: Some("Debugging support for various languages".to_string()),
                icon: Some("bug".to_string()),
            },
            ExtensionCategory {
                id: "formatters".to_string(),
                name: "Formatters".to_string(),
                description: Some("Code formatting tools".to_string()),
                icon: Some("format".to_string()),
            },
            ExtensionCategory {
                id: "linters".to_string(),
                name: "Linters".to_string(),
                description: Some("Code quality and linting tools".to_string()),
                icon: Some("check".to_string()),
            },
            ExtensionCategory {
                id: "snippets".to_string(),
                name: "Snippets".to_string(),
                description: Some("Code snippets and templates".to_string()),
                icon: Some("snippet".to_string()),
            },
            ExtensionCategory {
                id: "keymaps".to_string(),
                name: "Keymaps".to_string(),
                description: Some("Keyboard shortcuts and keybindings".to_string()),
                icon: Some("keyboard".to_string()),
            },
            ExtensionCategory {
                id: "other".to_string(),
                name: "Other".to_string(),
                description: Some("Other extensions".to_string()),
                icon: Some("extensions".to_string()),
            },
        ]
    }

    // ===== Featured Extensions =====

    /// Set featured extensions
    pub async fn set_featured_extensions(&self, extension_ids: Vec<String>) {
        let mut featured = self.featured_extensions.write().await;
        *featured = extension_ids;
        info!("Updated featured extensions");
    }

    /// Get featured extensions
    pub async fn get_featured_extensions(&self) -> Vec<String> {
        self.featured_extensions.read().await.clone()
    }

    // ===== Categories =====

    /// Get all categories
    pub async fn get_categories(&self) -> Vec<ExtensionCategory> {
        self.categories.read().await.clone()
    }

    /// Get category by ID
    pub async fn get_category(&self, category_id: &str) -> Option<ExtensionCategory> {
        self.categories.read().await.iter().find(|c| c.id == category_id).cloned()
    }

    // ===== Reviews =====

    /// Add review
    pub async fn add_review(
        &self, extension_id: String, review: ExtensionReview,
    ) -> Result<(), String> {
        let mut reviews = self.reviews.write().await;

        reviews.entry(extension_id.clone()).or_insert_with(Vec::new).push(review);

        // Update rating
        self.update_rating(&extension_id).await;

        // Emit event
        if let Err(e) = self.app_handle.emit("marketplace-review-added", &extension_id) {
            error!("Failed to emit review added event: {}", e);
        }

        info!("Added review for: {}", extension_id);

        Ok(())
    }

    /// Get reviews for extension
    pub async fn get_reviews(&self, extension_id: &str) -> Vec<ExtensionReview> {
        self.reviews.read().await.get(extension_id).cloned().unwrap_or_default()
    }

    /// Get reviews with pagination
    pub async fn get_reviews_paginated(
        &self, extension_id: &str, page: usize, page_size: usize,
    ) -> ReviewsPage {
        let all_reviews = self.get_reviews(extension_id).await;
        let total = all_reviews.len();

        let start = page * page_size;
        let end = std::cmp::min(start + page_size, total);

        let reviews = if start < total { all_reviews[start..end].to_vec() } else { Vec::new() };

        ReviewsPage {
            reviews,
            page,
            page_size,
            total,
            total_pages: total.div_ceil(page_size),
        }
    }

    // ===== Ratings =====

    /// Update rating for extension
    async fn update_rating(&self, extension_id: &str) {
        let reviews = self.reviews.read().await;

        if let Some(ext_reviews) = reviews.get(extension_id) {
            if ext_reviews.is_empty() {
                return;
            }

            let total: f64 = ext_reviews.iter().map(|r| r.rating as f64).sum();
            let average = total / ext_reviews.len() as f64;

            let mut ratings = self.ratings.write().await;
            ratings.insert(
                extension_id.to_string(),
                ExtensionRating {
                    average_rating: average,
                    total_ratings: ext_reviews.len(),
                    rating_distribution: Self::calculate_distribution(ext_reviews),
                },
            );
        }
    }

    fn calculate_distribution(reviews: &[ExtensionReview]) -> HashMap<u8, usize> {
        let mut distribution = HashMap::new();

        for review in reviews {
            *distribution.entry(review.rating).or_insert(0) += 1;
        }

        distribution
    }

    /// Get rating for extension
    pub async fn get_rating(&self, extension_id: &str) -> Option<ExtensionRating> {
        self.ratings.read().await.get(extension_id).cloned()
    }

    // ===== Recommendations =====

    /// Add recommendation
    pub async fn add_recommendation(&self, extension_id: String, recommended_ids: Vec<String>) {
        let mut recommendations = self.recommendations.write().await;
        recommendations.insert(extension_id.clone(), recommended_ids);

        info!("Added recommendations for: {}", extension_id);
    }

    /// Get recommendations
    pub async fn get_recommendations(&self, extension_id: &str) -> Vec<String> {
        self.recommendations.read().await.get(extension_id).cloned().unwrap_or_default()
    }

    /// Generate recommendations based on installed extensions
    pub async fn generate_recommendations(&self, installed_extensions: Vec<String>) -> Vec<String> {
        let recommendations = self.recommendations.read().await;
        let mut recommended = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for ext_id in &installed_extensions {
            if let Some(recs) = recommendations.get(ext_id) {
                for rec_id in recs {
                    if !installed_extensions.contains(rec_id) && !seen.contains(rec_id) {
                        recommended.push(rec_id.clone());
                        seen.insert(rec_id.clone());
                    }
                }
            }
        }

        recommended
    }

    // ===== Update Notifications =====

    /// Add update notification
    pub async fn add_update_notification(&self, notification: UpdateNotification) {
        let mut notifications = self.update_notifications.write().await;

        // Remove existing notification for same extension
        notifications.retain(|n| n.extension_id != notification.extension_id);

        notifications.push(notification.clone());

        // Emit event
        if let Err(e) = self.app_handle.emit("marketplace-update-available", &notification) {
            error!("Failed to emit update notification: {}", e);
        }

        info!("Added update notification for: {}", notification.extension_id);
    }

    /// Get all update notifications
    pub async fn get_update_notifications(&self) -> Vec<UpdateNotification> {
        self.update_notifications.read().await.clone()
    }

    /// Remove update notification
    pub async fn remove_update_notification(&self, extension_id: &str) {
        let mut notifications = self.update_notifications.write().await;
        notifications.retain(|n| n.extension_id != extension_id);

        info!("Removed update notification for: {}", extension_id);
    }

    /// Clear all update notifications
    pub async fn clear_update_notifications(&self) {
        let mut notifications = self.update_notifications.write().await;
        notifications.clear();

        info!("Cleared all update notifications");
    }

    // ===== Search & Filter =====

    /// Search extensions with filters
    #[allow(unused_variables)]
    pub async fn search_extensions(
        &self, query: &str, category: Option<String>, sort_by: SortBy,
    ) -> Vec<ExtensionSearchResult> {
        // This would integrate with the marketplace search
        // For now, return empty vec - actual implementation would call marketplace API
        Vec::new()
    }

    // ===== Cleanup =====

    /// Clear all data
    pub async fn clear_all_data(&self) {
        let mut featured = self.featured_extensions.write().await;
        featured.clear();

        let mut reviews = self.reviews.write().await;
        reviews.clear();

        let mut ratings = self.ratings.write().await;
        ratings.clear();

        let mut recommendations = self.recommendations.write().await;
        recommendations.clear();

        let mut notifications = self.update_notifications.write().await;
        notifications.clear();

        info!("[MarketplaceUI] Cleared all marketplace data");
    }
}

// ===== Type Definitions =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionCategory {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionReview {
    pub author: String,
    pub rating: u8,
    pub title: Option<String>,
    pub comment: Option<String>,
    pub created_at: String,
    pub helpful_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionRating {
    pub average_rating: f64,
    pub total_ratings: usize,
    pub rating_distribution: HashMap<u8, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewsPage {
    pub reviews: Vec<ExtensionReview>,
    pub page: usize,
    pub page_size: usize,
    pub total: usize,
    pub total_pages: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNotification {
    pub extension_id: String,
    pub extension_name: String,
    pub current_version: String,
    pub latest_version: String,
    pub change_log: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortBy {
    Relevance,
    Rating,
    Downloads,
    Name,
    Updated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionSearchResult {
    pub extension_id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub publisher: String,
    pub icon: Option<String>,
    pub rating: Option<f64>,
    pub downloads: usize,
    pub categories: Vec<String>,
}
