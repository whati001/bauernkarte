## Purpose

Lets registered users add photos of a store's products so other visitors
can see what's on offer before they show up in person.

## ADDED Requirements

### Requirement: Upload an image for a store's product
The system SHALL let a logged-in user upload an image tied to a specific
store-product listing. The system SHALL accept only decodable raster
images in JPEG, PNG, or WebP format, and SHALL reject any other content or
an unparseable upload.

#### Scenario: Valid image upload
- **WHEN** a logged-in user uploads a decodable JPEG, PNG, or WebP image
  for a store-product they can access
- **THEN** the system stores the image, unapproved, linked to that
  store-product

#### Scenario: Non-image or corrupt upload rejected
- **WHEN** a logged-in user uploads a file that is not a decodable raster
  image, or not one of the allowed formats
- **THEN** the system rejects the upload and stores nothing

#### Scenario: Oversized upload rejected
- **WHEN** a logged-in user's raw upload exceeds the configured size cap
- **THEN** the system rejects the upload before processing it

### Requirement: Uploaded images are normalized before storage
The system SHALL resize an accepted image so that neither its width nor
its height exceeds a fixed maximum (Full HD, 1920×1080), preserving
aspect ratio and never upscaling a smaller source, strip embedded
metadata (EXIF), and re-encode it before storing it, so stored images are
a predictable, reduced size.

#### Scenario: Large photo is downsized on upload
- **WHEN** a logged-in user uploads a photo exceeding 1920×1080
- **THEN** the system stores a resized, re-encoded version whose width and
  height are both at or under 1920×1080, with EXIF metadata removed

#### Scenario: Small photo is not upscaled
- **WHEN** a logged-in user uploads a photo smaller than 1920×1080 in both
  dimensions
- **THEN** the system stores it re-encoded at its original dimensions,
  without enlarging it

### Requirement: Serving an image respects approval and ownership
The system SHALL serve an image's raw bytes with the correct content type
derived from the stored image only if the image is approved, or if the
requester is the image's uploader.

#### Scenario: Approved image served to anyone
- **WHEN** any visitor requests an approved image by id
- **THEN** the system returns the image bytes with the correct content
  type

#### Scenario: Pending image visible only to its uploader
- **WHEN** a user who is not the uploader (or an anonymous visitor)
  requests an image that is not yet approved
- **THEN** the system does not return the image bytes

#### Scenario: Uploader can preview their own pending image
- **WHEN** the uploader requests their own not-yet-approved image
- **THEN** the system returns the image bytes with the correct content
  type

### Requirement: Image upload requires an authenticated session
The system SHALL reject image upload attempts from unauthenticated
requests.

#### Scenario: Anonymous upload attempt
- **WHEN** an unauthenticated visitor attempts to upload an image
- **THEN** the system rejects the request and stores nothing
