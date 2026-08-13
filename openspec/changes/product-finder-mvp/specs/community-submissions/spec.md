## Purpose

Lets registered users grow the dataset by adding stores and products they
know about, so the catalog covers more of the market than an admin alone
could enter.

## ADDED Requirements

### Requirement: Add a new store
The system SHALL let a logged-in user submit a new store with a name and a
position. The user SHALL either select an existing company for the store,
or indicate the store's name is also the company via a dedicated flag —
exactly one of the two SHALL be required per submission. When the flag is
used, the system SHALL create a new company (reusing the store's name, and
any provided description/homepage) before creating the store.

#### Scenario: New store linked to an existing company
- **WHEN** a logged-in user submits a store with a name, position, and an
  existing company selected
- **THEN** the system creates the store linked to that company

#### Scenario: New store that is its own company
- **WHEN** a logged-in user submits a store with the "this store is the
  company" flag set, instead of selecting an existing company
- **THEN** the system creates a new company using the store's name (and any
  provided description/homepage) and links the new store to it

#### Scenario: Submission missing both or providing both
- **WHEN** a logged-in user submits a store with neither an existing
  company selected nor the flag set, or with both
- **THEN** the system rejects the submission with a validation error and
  creates neither a company nor a store

### Requirement: Add a product to an existing store
The system SHALL let a logged-in user add a product listing to an existing
store, specifying a price. The user SHALL be able to select an existing
product or submit a new product (name, category, optional description).

#### Scenario: Add an existing product to a store
- **WHEN** a logged-in user selects an existing product and a price for one
  of their target store's listings
- **THEN** the system creates a store-product row linking that store and
  product at the given price

#### Scenario: Add a new product to a store
- **WHEN** a logged-in user submits a new product's name, category, and a
  price for the target store
- **THEN** the system creates the new product and a store-product row
  linking it to that store at the given price

### Requirement: All community submissions await moderation
Every company, store, product, and store-product row created through
these flows SHALL be created unapproved, and SHALL NOT appear in public
search or detail views until approved (see the content-moderation
capability).

#### Scenario: Newly submitted store is not publicly visible
- **WHEN** a logged-in user successfully submits a new store
- **THEN** the store does not appear in anonymous or other users' search
  results until an administrator approves it

### Requirement: Submission requires an authenticated session
The system SHALL reject store and product submissions from unauthenticated
requests.

#### Scenario: Anonymous submission attempt
- **WHEN** an unauthenticated visitor attempts to submit a new store or
  product
- **THEN** the system rejects the request and does not create any rows
