## Purpose

Lets any logged-in user correct or remove existing catalog data (a
company, store, product, store-product listing, or image) directly and
immediately, so the catalog stays accurate without waiting on the
original submitter or an admin.

## ADDED Requirements

### Requirement: Any logged-in user can edit an existing catalog entry
The system SHALL let any logged-in user — not only the row's original
creator — edit any field of an existing `company`, `store`, `product`,
`store_product` (including its price), or `image` (its description). The
edit SHALL take effect immediately: it SHALL NOT reset the row's
`approved` state and SHALL NOT require any further approval before being
publicly visible.

#### Scenario: Non-creator edits a catalog entry
- **WHEN** a logged-in user who did not create a store edits that store's
  opening hours
- **THEN** the system updates the store immediately and the new opening
  hours are publicly visible with no approval step

#### Scenario: Edit does not reset approval
- **WHEN** a logged-in user edits an already-`approved` store-product's
  price
- **THEN** the row remains `approved` and the new price is visible
  immediately, without returning to a pending state

#### Scenario: Anonymous edit attempt rejected
- **WHEN** an unauthenticated visitor attempts to edit any catalog entry
- **THEN** the system rejects the request and the entry is unchanged

#### Scenario: Editing a soft-deleted entry is rejected
- **WHEN** a logged-in user attempts to edit an entry that has been
  soft-deleted
- **THEN** the system rejects the edit until the entry is restored

### Requirement: Any logged-in user can remove an existing catalog entry
The system SHALL let any logged-in user soft-delete an existing
`company`, `store`, `product`, `store_product`, or `image` — marking it
`deleted` rather than permanently erasing it. A soft-deleted entry SHALL
immediately stop appearing in every public read path (search, detail,
listings, image serving) as if it were unapproved.

#### Scenario: Delete hides the entry immediately
- **WHEN** a logged-in user soft-deletes a store-product
- **THEN** that listing no longer appears in search results, store detail,
  or any other public read path immediately after the request completes

#### Scenario: Delete is reversible, not destructive
- **WHEN** a catalog entry is soft-deleted
- **THEN** its underlying row and data remain intact in the database,
  recoverable by clearing the `deleted` flag

#### Scenario: Anonymous delete attempt rejected
- **WHEN** an unauthenticated visitor attempts to delete any catalog entry
- **THEN** the system rejects the request and the entry is unchanged

#### Scenario: Deleting an already-deleted entry is rejected
- **WHEN** a logged-in user attempts to soft-delete an entry that is
  already soft-deleted
- **THEN** the system rejects the request as a no-op rather than deleting
  it again or erroring destructively

### Requirement: Editing scope excludes personal, non-catalog data
This capability SHALL NOT extend to `rating` rows (a rating remains
settable/removable only by the user who created it — see the ratings
capability) or to another user's account fields (name/email/password —
see the user-auth capability). It also SHALL NOT extend to `category` or
`rating_type`, which remain fixed, admin-managed taxonomies not editable
through this capability.

#### Scenario: Rating is not editable via catalog editing
- **WHEN** a logged-in user who did not create a given rating attempts to
  change or remove it through a catalog-editing action
- **THEN** the system rejects the request; only the rating's own creator
  can remove it, per the ratings capability

#### Scenario: Category is not user-editable
- **WHEN** any logged-in user attempts to edit a `category` or
  `rating_type` row
- **THEN** the system rejects the request

### Requirement: Edit/delete actions require an authenticated session and are rate-limited
The system SHALL apply the same per-IP rate limiting to catalog edit and
delete routes as it applies to other mutation routes (registration,
login, rating, image upload), to blunt automated vandalism given edits
are not moderation-gated.

#### Scenario: Excessive edit requests throttled
- **WHEN** a single IP address submits an excessive rate of edit or delete
  requests in a short window
- **THEN** the system throttles further requests from that IP, consistent
  with its treatment of other mutation routes
