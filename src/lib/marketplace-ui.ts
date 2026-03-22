import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// ===== Type Definitions =====

export interface ExtensionCategory {
  id: string;
  name: string;
  description?: string;
  icon?: string;
}

export interface ExtensionReview {
  author: string;
  rating: number;
  title?: string;
  comment?: string;
  createdAt: string;
  helpfulCount: number;
}

export interface ExtensionRating {
  averageRating: number;
  totalRatings: number;
  ratingDistribution: Record<number, number>;
}

export interface ReviewsPage {
  reviews: ExtensionReview[];
  page: number;
  pageSize: number;
  total: number;
  totalPages: number;
}

export interface UpdateNotification {
  extensionId: string;
  extensionName: string;
  currentVersion: string;
  latestVersion: string;
  changeLog?: string;
  createdAt: string;
}

export type SortBy = 'relevance' | 'rating' | 'downloads' | 'name' | 'updated';

export interface ExtensionSearchResult {
  extensionId: string;
  name: string;
  description: string;
  version: string;
  publisher: string;
  icon?: string;
  rating?: number;
  downloads: number;
  categories: string[];
}

export interface ReviewAddedEvent {
  extensionId: string;
}

export interface UpdateAvailableEvent {
  notification: UpdateNotification;
}

/**
 * Marketplace UI Manager
 */
export class MarketplaceUIManager {
  private onReviewAddedCallbacks: Array<(event: ReviewAddedEvent) => void> = [];
  private onUpdateAvailableCallbacks: Array<(event: UpdateAvailableEvent) => void> = [];

  constructor() {}

  /**
   * Initialize marketplace manager
   */
  async initialize(): Promise<void> {
    // Listen for events
    await listen<string>('marketplace-review-added', (event) => {
      this.notifyReviewAdded({ extensionId: event.payload });
    });

    await listen<UpdateNotification>('marketplace-update-available', (event) => {
      this.notifyUpdateAvailable({ notification: event.payload });
    });

    console.log('[Marketplace] Initialized');
  }

  // ===== Featured Extensions =====

  /**
   * Set featured extensions
   */
  async setFeaturedExtensions(extensionIds: string[]): Promise<void> {
    await invoke('set_featured_extensions', { extensionIds });
  }

  /**
   * Get featured extensions
   */
  async getFeaturedExtensions(): Promise<string[]> {
    return await invoke<string[]>('get_featured_extensions');
  }

  // ===== Categories =====

  /**
   * Get all extension categories
   */
  async getCategories(): Promise<ExtensionCategory[]> {
    return await invoke<ExtensionCategory[]>('get_extension_categories');
  }

  /**
   * Get extension category by ID
   */
  async getCategory(categoryId: string): Promise<ExtensionCategory | null> {
    return await invoke<ExtensionCategory | null>('get_extension_category', { categoryId });
  }

  // ===== Reviews =====

  /**
   * Add extension review
   */
  async addReview(extensionId: string, review: ExtensionReview): Promise<void> {
    await invoke('add_extension_review', { extensionId, review });
  }

  /**
   * Get extension reviews
   */
  async getReviews(extensionId: string): Promise<ExtensionReview[]> {
    return await invoke<ExtensionReview[]>('get_extension_reviews', { extensionId });
  }

  /**
   * Get extension reviews with pagination
   */
  async getReviewsPaginated(
    extensionId: string,
    page: number = 0,
    pageSize: number = 10
  ): Promise<ReviewsPage> {
    return await invoke<ReviewsPage>('get_extension_reviews_paginated', {
      extensionId,
      page,
      pageSize,
    });
  }

  // ===== Ratings =====

  /**
   * Get extension rating
   */
  async getRating(extensionId: string): Promise<ExtensionRating | null> {
    return await invoke<ExtensionRating | null>('get_extension_rating', { extensionId });
  }

  // ===== Recommendations =====

  /**
   * Add extension recommendations
   */
  async addRecommendations(extensionId: string, recommendedIds: string[]): Promise<void> {
    await invoke('add_extension_recommendations', { extensionId, recommendedIds });
  }

  /**
   * Get extension recommendations
   */
  async getRecommendations(extensionId: string): Promise<string[]> {
    return await invoke<string[]>('get_extension_recommendations', { extensionId });
  }

  /**
   * Generate recommendations based on installed extensions
   */
  async generateRecommendations(installedExtensions: string[]): Promise<string[]> {
    return await invoke<string[]>('generate_recommendations', { installedExtensions });
  }

  // ===== Update Notifications =====

  /**
   * Add update notification
   */
  async addUpdateNotification(notification: UpdateNotification): Promise<void> {
    await invoke('add_update_notification', { notification });
  }

  /**
   * Get all update notifications
   */
  async getUpdateNotifications(): Promise<UpdateNotification[]> {
    return await invoke<UpdateNotification[]>('get_update_notifications');
  }

  /**
   * Remove update notification
   */
  async removeUpdateNotification(extensionId: string): Promise<void> {
    await invoke('remove_update_notification', { extensionId });
  }

  /**
   * Clear all update notifications
   */
  async clearUpdateNotifications(): Promise<void> {
    await invoke('clear_update_notifications');
  }

  // ===== Search & Filter =====

  /**
   * Search extensions with filters
   */
  async searchExtensions(
    query: string,
    category?: string,
    sortBy: SortBy = 'relevance'
  ): Promise<ExtensionSearchResult[]> {
    return await invoke<ExtensionSearchResult[]>('search_marketplace_extensions', {
      query,
      category: category || null,
      sortBy,
    });
  }

  // ===== Utility Methods =====

  /**
   * Clear all marketplace data
   */
  async clearMarketplaceData(): Promise<void> {
    await invoke('clear_marketplace_data');
  }

  /**
   * Create review helper
   */
  createReview(
    author: string,
    rating: number,
    options: {
      title?: string;
      comment?: string;
      helpfulCount?: number;
    } = {}
  ): ExtensionReview {
    return {
      author,
      rating,
      title: options.title,
      comment: options.comment,
      createdAt: new Date().toISOString(),
      helpfulCount: options.helpfulCount || 0,
    };
  }

  /**
   * Create update notification helper
   */
  createUpdateNotification(
    extensionId: string,
    extensionName: string,
    currentVersion: string,
    latestVersion: string,
    changeLog?: string
  ): UpdateNotification {
    return {
      extensionId,
      extensionName,
      currentVersion,
      latestVersion,
      changeLog,
      createdAt: new Date().toISOString(),
    };
  }

  /**
   * Format rating as stars
   */
  formatRatingStars(rating: number): string {
    const fullStars = Math.floor(rating);
    const hasHalfStar = rating % 1 >= 0.5;
    const emptyStars = 5 - fullStars - (hasHalfStar ? 1 : 0);

    return (
      '★'.repeat(fullStars) +
      (hasHalfStar ? '½' : '') +
      '☆'.repeat(emptyStars)
    );
  }

  /**
   * Get rating summary text
   */
  getRatingSummary(rating: ExtensionRating): string {
    const avg = rating.averageRating.toFixed(1);
    const stars = this.formatRatingStars(rating.averageRating);
    return `${stars} ${avg} (${rating.totalRatings} ${rating.totalRatings === 1 ? 'rating' : 'ratings'})`;
  }

  /**
   * Calculate rating percentage
   */
  getRatingPercentage(rating: ExtensionRating, stars: number): number {
    const count = rating.ratingDistribution[stars] || 0;
    return rating.totalRatings > 0 ? (count / rating.totalRatings) * 100 : 0;
  }

  /**
   * Format download count
   */
  formatDownloads(count: number): string {
    if (count >= 1000000) {
      return `${(count / 1000000).toFixed(1)}M`;
    } else if (count >= 1000) {
      return `${(count / 1000).toFixed(1)}K`;
    }
    return count.toString();
  }

  /**
   * Parse semantic version
   */
  parseVersion(version: string): { major: number; minor: number; patch: number } {
    const parts = version.split('.').map((p) => parseInt(p, 10));
    return {
      major: parts[0] || 0,
      minor: parts[1] || 0,
      patch: parts[2] || 0,
    };
  }

  /**
   * Compare versions
   */
  compareVersions(v1: string, v2: string): number {
    const ver1 = this.parseVersion(v1);
    const ver2 = this.parseVersion(v2);

    if (ver1.major !== ver2.major) {
      return ver1.major - ver2.major;
    }
    if (ver1.minor !== ver2.minor) {
      return ver1.minor - ver2.minor;
    }
    return ver1.patch - ver2.patch;
  }

  /**
   * Check if update is available
   */
  isUpdateAvailable(currentVersion: string, latestVersion: string): boolean {
    return this.compareVersions(currentVersion, latestVersion) < 0;
  }

  /**
   * Get update type
   */
  getUpdateType(currentVersion: string, latestVersion: string): 'major' | 'minor' | 'patch' | 'none' {
    const current = this.parseVersion(currentVersion);
    const latest = this.parseVersion(latestVersion);

    if (latest.major > current.major) {
      return 'major';
    } else if (latest.minor > current.minor) {
      return 'minor';
    } else if (latest.patch > current.patch) {
      return 'patch';
    }
    return 'none';
  }

  /**
   * Filter extensions by category
   */
  filterByCategory(
    extensions: ExtensionSearchResult[],
    categoryId: string
  ): ExtensionSearchResult[] {
    return extensions.filter((ext) => ext.categories.includes(categoryId));
  }

  /**
   * Sort extensions
   */
  sortExtensions(
    extensions: ExtensionSearchResult[],
    sortBy: SortBy
  ): ExtensionSearchResult[] {
    const sorted = [...extensions];

    switch (sortBy) {
      case 'rating':
        sorted.sort((a, b) => (b.rating || 0) - (a.rating || 0));
        break;
      case 'downloads':
        sorted.sort((a, b) => b.downloads - a.downloads);
        break;
      case 'name':
        sorted.sort((a, b) => a.name.localeCompare(b.name));
        break;
      case 'updated':
        // Would need update timestamp in ExtensionSearchResult
        break;
      case 'relevance':
      default:
        // Keep original order (from search relevance)
        break;
    }

    return sorted;
  }

  /**
   * Group extensions by category
   */
  groupByCategory(extensions: ExtensionSearchResult[]): Map<string, ExtensionSearchResult[]> {
    const grouped = new Map<string, ExtensionSearchResult[]>();

    for (const ext of extensions) {
      for (const category of ext.categories) {
        if (!grouped.has(category)) {
          grouped.set(category, []);
        }
        grouped.get(category)!.push(ext);
      }
    }

    return grouped;
  }

  /**
   * Get popular extensions (high downloads and ratings)
   */
  getPopularExtensions(extensions: ExtensionSearchResult[], limit: number = 10): ExtensionSearchResult[] {
    return extensions
      .filter((ext) => ext.rating && ext.rating >= 4.0 && ext.downloads >= 1000)
      .sort((a, b) => b.downloads - a.downloads)
      .slice(0, limit);
  }

  /**
   * Get trending extensions (recent high activity)
   */
  getTrendingExtensions(extensions: ExtensionSearchResult[], limit: number = 10): ExtensionSearchResult[] {
    // Would need additional metadata like recent download velocity
    // For now, use downloads as proxy
    return this.getPopularExtensions(extensions, limit);
  }

  // ===== Event Subscriptions =====

  /**
   * Subscribe to review added events
   */
  onReviewAdded(callback: (event: ReviewAddedEvent) => void): () => void {
    this.onReviewAddedCallbacks.push(callback);

    return () => {
      const index = this.onReviewAddedCallbacks.indexOf(callback);
      if (index > -1) {
        this.onReviewAddedCallbacks.splice(index, 1);
      }
    };
  }

  /**
   * Subscribe to update available events
   */
  onUpdateAvailable(callback: (event: UpdateAvailableEvent) => void): () => void {
    this.onUpdateAvailableCallbacks.push(callback);

    return () => {
      const index = this.onUpdateAvailableCallbacks.indexOf(callback);
      if (index > -1) {
        this.onUpdateAvailableCallbacks.splice(index, 1);
      }
    };
  }

  // ===== Internal Methods =====

  private notifyReviewAdded(event: ReviewAddedEvent): void {
    for (const callback of this.onReviewAddedCallbacks) {
      try {
        callback(event);
      } catch (error) {
        console.error('[Marketplace] Error in review added callback:', error);
      }
    }
  }

  private notifyUpdateAvailable(event: UpdateAvailableEvent): void {
    for (const callback of this.onUpdateAvailableCallbacks) {
      try {
        callback(event);
      } catch (error) {
        console.error('[Marketplace] Error in update available callback:', error);
      }
    }
  }
}

// Export singleton instance
export const marketplaceUI = new MarketplaceUIManager();
