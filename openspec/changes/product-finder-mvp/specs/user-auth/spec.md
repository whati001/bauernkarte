## Purpose

Lets visitors create an account and authenticate so they can contribute
data, while keeping anonymous browsing fully read-only.

## ADDED Requirements

### Requirement: Registration
The system SHALL let a visitor register with a name, email, and password.
The system SHALL validate that the email is a plausible email address and
the password is at least 8 characters, on both client and server. On
success the system SHALL store the password as an Argon2id hash (never
plaintext or a fast unsalted hash), create the account with `verified =
false`, and log the new user in immediately.

#### Scenario: Successful registration
- **WHEN** a visitor submits a valid name, unique email, and password of at
  least 8 characters
- **THEN** the system creates the account with an Argon2id password hash
  and `verified = false`, and starts a logged-in session for that user

#### Scenario: Registration rejects weak or invalid input
- **WHEN** a visitor submits a password shorter than 8 characters or a
  malformed email
- **THEN** the system rejects the submission and returns a validation error
  without creating an account

#### Scenario: Registration rejects duplicate email
- **WHEN** a visitor submits an email already registered (case-insensitive)
- **THEN** the system rejects the submission without creating a second
  account

### Requirement: Login and logout
The system SHALL let a registered user log in with email and password,
establishing a server-side session identified by an HttpOnly, Secure,
SameSite=Lax cookie. The system SHALL let a logged-in user log out,
destroying that session.

#### Scenario: Successful login
- **WHEN** a user submits the correct email and password
- **THEN** the system establishes a session for that user and the client
  reflects a logged-in state

#### Scenario: Failed login
- **WHEN** a user submits an email/password combination that does not match
  a stored account
- **THEN** the system rejects the login and no session is created

#### Scenario: Logout ends the session
- **WHEN** a logged-in user logs out
- **THEN** the system destroys their session and subsequent requests are
  treated as anonymous

### Requirement: Account editing
The system SHALL let a logged-in user view and update their own name and
email, and change their password. Changing the password SHALL require
providing the current password.

#### Scenario: Update profile fields
- **WHEN** a logged-in user submits a new name and/or email
- **THEN** the system updates the account with the new values

#### Scenario: Change password with correct current password
- **WHEN** a logged-in user submits their correct current password and a
  new password meeting the length requirement
- **THEN** the system updates the stored Argon2id hash to the new password

#### Scenario: Change password with incorrect current password
- **WHEN** a logged-in user submits an incorrect current password
- **THEN** the system rejects the change and the stored password is
  unchanged

### Requirement: Anonymous access is read-only
The system SHALL treat every state-changing route (submissions, ratings,
image uploads, account edits) as requiring an authenticated session, and
SHALL reject unauthenticated attempts.

#### Scenario: Unauthenticated mutation attempt
- **WHEN** a request without a valid session attempts a state-changing
  action (e.g. submitting a store, rating, or image)
- **THEN** the system rejects the request as unauthorized and does not
  perform the action
