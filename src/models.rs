//! Types shared by the server functions and the components that render
//! them — everything here crosses the wire as JSON. Row structs that only
//! the database layer needs live in `server::db` instead.

use serde::{Deserialize, Serialize};

use crate::i18n::Locale;

/// The signed-in visitor, as much of them as the UI needs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub admin: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionInfo {
    pub user: Option<SessionUser>,
    pub locale: Locale,
}

/// One weekday's opening hours. `day` is ISO 8601 (1 = Monday .. 7 =
/// Sunday); stored as a sparse JSONB array where a missing day is closed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DayHours {
    pub day: i16,
    pub open: String,
    pub close: String,
}

/// One product a store carries, as summarised for search results and map
/// pins. Always one of a store's top 5 by rating.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductSummary {
    pub name: String,
    pub icon: Option<String>,
    pub rating_count: i64,
}

/// One matching store — the same rows drive the results list and the
/// map's pins. `distance_m` is `None` until geolocation resolves.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoreSearchResult {
    pub id: i64,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub distance_m: Option<f64>,
    pub products: Vec<ProductSummary>,
    pub product_total: i64,
}

/// An approved catalog product: the filter `<select>`, the navbar's
/// quick-pick row, the search suggestions and the "existing product"
/// pickers in the forms.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatalogProduct {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
}

impl CatalogProduct {
    pub fn icon_or_default(&self) -> &str {
        self.icon.as_deref().unwrap_or("📦")
    }
}

/// The shop's 1–5 star rating: average and count over every review, plus
/// the viewer's own when they've left one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ReviewSummary {
    pub average: Option<f64>,
    pub count: i64,
    pub mine: Option<i16>,
}

/// One offer ("this shop sells this product") on the detail panel.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfferDetail {
    pub store_product_id: i64,
    pub product_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    /// `None` = available all year.
    pub seasonal_months: Option<Vec<i16>>,
    /// Count of per-offer "UP" hearts.
    pub hearts: i64,
    pub viewer_has_rated_up: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageSummary {
    pub id: i64,
    pub description: Option<String>,
}

/// Everything the store info panel shows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoreDetail {
    pub id: i64,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    /// Sparse; empty means "not specified" and the hours line is hidden.
    pub openinghours: Vec<DayHours>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub owner_name: Option<String>,
    pub owner_since: Option<i16>,
    pub owner_bio: Option<String>,
    /// The newest approved owner portrait, for the avatar.
    pub owner_image_id: Option<i64>,
    pub offers: Vec<OfferDetail>,
    /// Photos of the place (not portraits), oldest first. The first is the
    /// header image.
    pub photos: Vec<ImageSummary>,
    pub review: ReviewSummary,
}

/// The fields of a store the create/edit form handles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct StoreFields {
    pub name: String,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub openinghours: Vec<DayHours>,
    pub address: String,
    pub phone: String,
    pub owner_name: String,
    /// Kept as typed text so the form can say what's wrong with it.
    pub owner_since: String,
    pub owner_bio: String,
}

/// Which product an offer is for: one from the catalog, or a new one to
/// submit alongside it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProductChoice {
    Existing(i64),
    New { name: String, description: String },
}

/// One offer as entered in a form (new store, or "add product").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfferInput {
    pub product: ProductChoice,
    pub seasonal_months: Option<Vec<i16>>,
}

/// An existing offer, loaded into the seasonality form.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfferEdit {
    pub store_product_id: i64,
    pub store_id: i64,
    pub product_name: String,
    pub seasonal_months: Option<Vec<i16>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductEdit {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
}

/// One of the viewer's own not-yet-approved submissions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PendingItem {
    pub kind: PendingKind,
    pub id: i64,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PendingKind {
    Store,
    Product,
    Offer,
    Image,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountData {
    pub name: String,
    pub email: String,
    pub pending: Vec<PendingItem>,
}

/// The single `site_info` row behind the Impressum.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SiteInfo {
    pub operator_name: String,
    pub street: String,
    pub postal_code: String,
    pub city: String,
    pub country: String,
    pub email: String,
    pub phone: String,
    pub vat_id: String,
    pub register_number: String,
    pub responsible: String,
    pub purpose: String,
}

impl SiteInfo {
    /// An Impressum with no operator named isn't a short one, it's an
    /// empty one.
    pub fn is_configured(&self) -> bool {
        !self.operator_name.trim().is_empty()
    }

    pub fn postal_city(&self) -> String {
        [self.postal_code.trim(), self.city.trim()]
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

// ---------------------------------------------------------------------
// Admin
// ---------------------------------------------------------------------

/// The four moderated tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Entity {
    Store,
    Product,
    /// `store_product` — "this shop sells this product".
    Offer,
    Image,
}

impl Entity {
    pub const ALL: [Entity; 4] = [Entity::Store, Entity::Product, Entity::Offer, Entity::Image];

    /// URL segment under `/admin/`.
    pub fn slug(self) -> &'static str {
        match self {
            Entity::Store => "stores",
            Entity::Product => "products",
            Entity::Offer => "offers",
            Entity::Image => "images",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|e| e.slug() == slug)
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Entity::Store => "admin-nav-stores",
            Entity::Product => "admin-nav-products",
            Entity::Offer => "admin-nav-offers",
            Entity::Image => "admin-nav-images",
        }
    }

    pub fn blurb_key(self) -> &'static str {
        match self {
            Entity::Store => "admin-blurb-stores",
            Entity::Product => "admin-blurb-products",
            Entity::Offer => "admin-blurb-offers",
            Entity::Image => "admin-blurb-images",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueueTab {
    Pending,
    Changes,
    Deleted,
}

impl QueueTab {
    pub const ALL: [QueueTab; 3] = [QueueTab::Pending, QueueTab::Changes, QueueTab::Deleted];

    pub fn key(self) -> &'static str {
        match self {
            QueueTab::Pending => "pending",
            QueueTab::Changes => "changes",
            QueueTab::Deleted => "deleted",
        }
    }

    pub fn from_key(key: &str) -> Self {
        match key {
            "changes" => QueueTab::Changes,
            "deleted" => QueueTab::Deleted,
            _ => QueueTab::Pending,
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            QueueTab::Pending => "admin-tab-pending",
            QueueTab::Changes => "admin-tab-changes",
            QueueTab::Deleted => "admin-tab-deleted",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct QueueCounts {
    pub pending: i64,
    pub changes: i64,
    pub deleted: i64,
}

impl QueueCounts {
    pub fn for_tab(&self, tab: QueueTab) -> i64 {
        match tab {
            QueueTab::Pending => self.pending,
            QueueTab::Changes => self.changes,
            QueueTab::Deleted => self.deleted,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueueRow {
    pub id: i64,
    pub title: String,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    pub at_human: String,
    pub at_iso: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDiff {
    pub field: String,
    pub old: String,
    pub new: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChangeRow {
    pub log_id: i64,
    pub title: String,
    pub author: Option<String>,
    pub at_human: String,
    pub at_iso: String,
    pub diff: Vec<FieldDiff>,
}

/// One admin queue page: the visible tab's rows plus every tab's count.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QueuePage {
    pub counts: QueueCounts,
    pub rows: Vec<QueueRow>,
    pub changes: Vec<ChangeRow>,
}

/// The admin rail's pending badge per section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RailCount {
    pub entity: Entity,
    pub pending: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdminUserRow {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub admin: bool,
    pub contributions: i64,
    pub created_human: String,
    /// False for the viewing admin's own row, the last admin and the seed
    /// account — acting on any of those locks the door from inside.
    pub protected: bool,
}
