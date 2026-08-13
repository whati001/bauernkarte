## Purpose

Keeps unreviewed, user-submitted data out of public view until an
administrator vets it, keeps soft-deleted data out of public view until
restored, and gives an administrator enough of a paper trail — pending
submissions, and a log of every edit and delete — to review, revert, or
restore content without any in-app moderation UI in v1.

## ADDED Requirements

### Requirement: User-submitted content defaults to unapproved
Every company, store, product, store-product, and image row created via a
community submission or upload flow SHALL be created with `approved =
false`.

#### Scenario: New submission is unapproved by default
- **WHEN** a logged-in user's submission (store, company, product,
  store-product, or image) is created
- **THEN** the created row has `approved = false`

### Requirement: Public reads exclude unapproved or soft-deleted content
Every read path exposed to visitors — search results, map markers, store
detail, product listings, and image serving — SHALL exclude company,
store, product, store-product, and image rows that are not approved, or
that have been soft-deleted (see the catalog-editing capability), except
as permitted by the submitter's own pending-items view or the
image-upload capability's owner-preview rule.

#### Scenario: Unapproved product hidden from other users
- **WHEN** a store-product submitted by user A is not yet approved
- **THEN** the listing does not appear to user B or to anonymous visitors
  in search, detail, or listings

#### Scenario: Soft-deleted entry hidden from all users
- **WHEN** a company, store, product, store-product, or image has been
  soft-deleted
- **THEN** it does not appear to any visitor, including the user who
  deleted it, in any public read path

### Requirement: Approval, rejection, revert, and restore are performed outside the application
The system SHALL NOT expose an in-app interface for approving or rejecting
submissions, reverting an edit, or restoring a soft-deleted entry in v1;
all of these SHALL be performed as direct database operations (updating
the `approved`/`deleted` flags, or reading the audit log to recover a
prior value).

#### Scenario: No admin approval endpoint
- **WHEN** any user, including a logged-in one, uses the application
- **THEN** no route or UI control is offered to change another user's
  submission's `approved` flag, revert another user's edit, or restore a
  soft-deleted entry

### Requirement: Every catalog edit and delete is recorded in an audit log
The system SHALL record every catalog edit and every soft-delete (see the
catalog-editing capability) in an audit log capturing at minimum the
affected entity, the action taken, the value before and after the change
(before-only for a delete), who made the change, and when — sufficient
for an administrator to revert a bad edit or restore a deleted entry via
direct database access.

#### Scenario: Edit is logged
- **WHEN** a logged-in user edits an existing catalog entry
- **THEN** the system records an audit log entry with the prior and new
  values, the editing user, and a timestamp

#### Scenario: Delete is logged
- **WHEN** a logged-in user soft-deletes a catalog entry
- **THEN** the system records an audit log entry with the entry's prior
  value, the deleting user, and a timestamp

### Requirement: Submitters can see their own pending submissions
The system SHALL let a logged-in user view a list of their own
not-yet-approved submissions (companies, stores, products, store-products,
images) they created, separate from the public catalog.

#### Scenario: User views their pending submissions
- **WHEN** a logged-in user who has submitted unapproved content opens
  their account page
- **THEN** the system shows them the list of their own submissions that are
  not yet approved

#### Scenario: Pending list excludes other users' unapproved content
- **WHEN** a logged-in user opens their pending-submissions list
- **THEN** the list contains only rows they themselves created, not other
  users' unapproved submissions
