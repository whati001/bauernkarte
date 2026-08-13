## Purpose

Lets registered users signal which store-product listings are good, giving
other visitors a lightweight quality signal that updates in real time
without moderation delay.

## ADDED Requirements

### Requirement: Toggle a rating on a store's product
The system SHALL let a logged-in user set a rating of a given type (default
`UP`, shown as ❤️) on a specific store's product listing, and SHALL let
them remove their own rating. A user SHALL have at most one rating of a
given type on a given store-product at a time.

#### Scenario: First rating from a user
- **WHEN** a logged-in user rates a store-product they have not previously
  rated with that rating type
- **THEN** the system records the rating and the displayed count for that
  type increases by one

#### Scenario: Re-rating is idempotent, not stacking
- **WHEN** a logged-in user rates the same store-product with the same
  rating type they already rated
- **THEN** the system does not create a second rating and the count is
  unchanged

#### Scenario: Removing a rating
- **WHEN** a logged-in user removes their own existing rating on a
  store-product
- **THEN** the system deletes that rating and the displayed count for that
  type decreases by one

#### Scenario: Removing another user's rating is rejected
- **WHEN** a logged-in user attempts to remove a rating they did not create
- **THEN** the system rejects the request and the rating is unchanged

### Requirement: Ratings are visible immediately
Ratings SHALL NOT be subject to the moderation/approval workflow — a
rating on an approved store-product SHALL be reflected in counts shown to
all visitors as soon as it is recorded or removed.

#### Scenario: Rating shown without approval delay
- **WHEN** a logged-in user rates an approved store-product
- **THEN** the updated count is visible to anonymous visitors immediately,
  with no separate approval step

### Requirement: Rating action requires an authenticated session
The system SHALL reject rating and un-rating attempts from unauthenticated
requests.

#### Scenario: Anonymous rating attempt
- **WHEN** an unauthenticated visitor attempts to rate a store-product
- **THEN** the system rejects the request and no rating is recorded
