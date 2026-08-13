## Purpose

Lets any visitor drill into a single store to see what it sells, at what
price and rating, with photos and a way to get directions.

## ADDED Requirements

### Requirement: Store detail reachable by selection or direct link
The system SHALL display a store's detail view when a visitor selects it
from search results or a map marker, and SHALL also serve the same detail
view directly at a shareable per-store URL.

#### Scenario: Selecting a result opens detail
- **WHEN** a visitor selects a store from the results list or clicks its
  map marker
- **THEN** the detail view for that store replaces the search panel

#### Scenario: Direct link opens detail without prior search
- **WHEN** a visitor opens a store's direct URL with no prior search
- **THEN** the system renders the page with that store's detail view
  pre-loaded

### Requirement: Store detail content
The store detail view SHALL show the store's company name, description,
and homepage (if set), the store's opening hours, a link that opens the
store's location in Google Maps, and the full list of the store's approved
products with price and rating count per product.

#### Scenario: Full detail rendered
- **WHEN** a visitor opens a store's detail view
- **THEN** the response includes company info, opening hours, a Google
  Maps link built from the store's coordinates, and each approved product
  with its price and rating count

### Requirement: Rating counts grouped by rating type
Each product's rating display SHALL show a count per `rating_type` (e.g.
`10 ❤️` for the `UP` type), rendered generically so additional rating
types do not require a template rewrite.

#### Scenario: Product with ratings of one type
- **WHEN** a store's product has ratings only of the `UP` type
- **THEN** the detail view shows that type's count next to its icon, and no
  other type is shown

### Requirement: Product image gallery
Each product in the detail view SHALL display its approved images as
thumbnails, with a way to view an enlarged version.

#### Scenario: Product has approved images
- **WHEN** a store's product has one or more approved images
- **THEN** the detail view shows thumbnails for each, and selecting one
  shows an enlarged view

### Requirement: Navigating back to search preserves prior filters
From the detail view, the system SHALL provide a way back to the previous
search results without the visitor re-entering their filters, and SHALL
support the equivalent action via the Escape key.

#### Scenario: Back action restores prior search
- **WHEN** a visitor opens detail from a filtered search result and then
  triggers "back" (button or Escape)
- **THEN** the panel returns to the search view with the same filters and
  results as before

### Requirement: Switching selection while in detail view
The system SHALL allow selecting a different store or product while
already viewing detail, replacing the panel directly without requiring the
visitor to return to search first.

#### Scenario: Selecting another marker while in detail
- **WHEN** a visitor is viewing one store's detail and selects a different
  map marker or result
- **THEN** the panel updates to the newly selected store's detail directly
