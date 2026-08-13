## Purpose

Lets any visitor, without logging in, discover which approved stores near a
location carry a given product or category on a Google-Maps-style map.

## ADDED Requirements

### Requirement: Anonymous map-first search
The system SHALL allow unauthenticated visitors to browse approved stores
on a map and as a results list, without requiring login.

#### Scenario: Anonymous visitor loads the search page
- **WHEN** an unauthenticated visitor opens the search page
- **THEN** the system returns a map centered per the default-center rule and
  a results list of approved stores within the default search radius

### Requirement: Default map center with geolocation fallback
The system SHALL center the initial map view on the visitor's browser
geolocation when available, and SHALL fall back to Austria's geographic
centroid (≈47.5162° N, 14.5501° E) with a visible "location unavailable"
indicator when geolocation is denied or unavailable. The radius control
SHALL default to 5 km once geolocation has actually resolved to a real
position; while geolocation is unresolved or unavailable, the radius
control SHALL be hidden entirely (a radius drawn around an arbitrary
country-centroid fallback implies a precision the system doesn't have)
and the search SHALL instead run at its full maximum radius (100 km).

#### Scenario: Geolocation permission granted
- **WHEN** the visitor's browser reports a geolocation position on page load
- **THEN** the map centers on that position, the radius control becomes
  visible, and the search radius defaults to 5 km

#### Scenario: Geolocation permission denied
- **WHEN** the visitor's browser denies or does not support geolocation
- **THEN** the map centers on Austria's geographic centroid, a "location
  unavailable" note is shown, the radius control is hidden, and search
  runs at the full 100 km radius rather than a location-anchored default

### Requirement: Category, product, and distance filtering
The system SHALL let a visitor filter search results by category, by a
specific product (product options cascade from the selected category), and
by a distance radius around the search location. The distance radius
SHALL be capped at 100 km, enforced both in the filter control and by the
server, regardless of what value a client sends.

#### Scenario: Filter by category cascades product options
- **WHEN** a visitor selects a category
- **THEN** the product filter is populated with only products belonging to
  that category

#### Scenario: Distance radius is capped at 100 km
- **WHEN** a visitor sets or a client requests a distance radius greater
  than 100 km
- **THEN** the system clamps the effective search radius to 100 km

### Requirement: Search radius shown on the map
While the radius control is visible (geolocation resolved — see the
default-center requirement), the system SHALL draw the current search
radius as a circle on the map, centered on the search location, that
updates live as the visitor adjusts the radius control.

#### Scenario: Circle reflects the current radius
- **WHEN** a visitor with resolved geolocation adjusts the distance radius
  control
- **THEN** the circle drawn on the map updates to match the new radius
  without a page reload

#### Scenario: Filter by product and distance
- **WHEN** a visitor selects a product and sets a distance radius
- **THEN** the results list and map markers show only approved stores that
  carry that product within the radius, ordered by distance

### Requirement: Search results include only approved content
The system SHALL exclude stores, companies, products, and store-product
listings that are not approved from search results and map markers.

#### Scenario: Unapproved store excluded
- **WHEN** a store, its company, or all of its store-product listings are
  not yet approved
- **THEN** that store does not appear in search results or as a map marker
  for anonymous or logged-in searches

### Requirement: Search results carry minimal per-store summary data
Each search result and map marker SHALL include at minimum the store's id,
name, coordinates, distance from the search location, a best-matching
product name, and that product's rating count, without requiring a
separate request per store.

#### Scenario: Result list rendered from a single search request
- **WHEN** a visitor runs or changes a search (category/product/distance
  change)
- **THEN** the system returns updated results and markers in the same
  response, without additional per-store requests
