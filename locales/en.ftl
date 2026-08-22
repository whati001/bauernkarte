# English translation. Keys must match locales/de.ftl exactly — see
# src/i18n.rs's startup check, which fails loudly on drift instead of
# silently falling back key-by-key.

## Navbar
nav-brand = BauernKarte
nav-tagline = Buy local, support farmers
nav-login = Login
nav-logout = Logout
nav-account = Account
nav-account-of = Account of { $name }
nav-new-store = New Store
nav-search-label = Search category or product
nav-search-placeholder = Category or product…
nav-search-no-matches = No matching category or product
nav-search-clear = Reset search
nav-search-toggle = Open or close search
nav-products-label = Popular products
nav-products-all = All products

## Search sidebar
search-filter-heading = Filter
search-category = Category
search-product = Product
search-all = All
search-results-count = { $count ->
    [one] { $count } result
   *[other] { $count } results
}
search-no-results = No stores match this selection.
search-more = more
search-location-unavailable = Location unavailable
search-pick-on-map = 👆 Click the map to choose a location
search-location-picked = 📍 Location picked
search-use-my-location = Use my location

## Common actions
action-save = Save
action-delete = Delete
action-cancel = Cancel
action-edit = Edit
action-back = Back
action-back-to-search = Back to search

## Auth
auth-login-heading = Login
auth-register-heading = Register
auth-email = Email
auth-password = Password
auth-password-hint = Password (min. 12 characters)
auth-name = Name
auth-no-account = No account yet?
auth-have-account = Already registered?
auth-welcome-back = Welcome back, { $name }!
auth-register-success = Welcome, { $name }! Your registration was successful.

## Account page
account-heading = Account
account-change-password = Change password
account-current-password = Current password
account-new-password = New password
account-pending-heading = My submissions (pending review)
account-pending-empty = No pending submissions.
account-profile-saved = Your account details were saved.
account-password-changed = Your password was changed.

## Store detail
detail-open-in-maps = Open in Google Maps
detail-add-product = Add product
detail-add-image = Add image
detail-rate = Rate
detail-unrate = Remove rating
detail-no-products = No products for this store yet.
detail-seasonal-availability = Seasonal availability
detail-location = Location
detail-rating-label = Rating
detail-photos = Photos
detail-other-stores = More stores from this company
detail-get-directions = Get directions
detail-company = Company
detail-store = Store
detail-products = Products
detail-season = Season
detail-category = Category
detail-product-count = { $count ->
    [one] { $count } product
   *[other] { $count } products
  }
detail-image-count = { $count ->
    [0] No photos
    [one] { $count } photo
   *[other] { $count } photos
  }
detail-hero-art-alt = Illustration of a farm with the products on offer
opening-hours-closed = Closed

## Weekdays (opening-hours grid — store_form.html, sidebar_detail.html)
weekday-mon = Mon
weekday-tue = Tue
weekday-wed = Wed
weekday-thu = Thu
weekday-fri = Fri
weekday-sat = Sat
weekday-sun = Sun

## Months (seasonality grid — product_form.html, sidebar_detail.html)
month-jan = Jan
month-feb = Feb
month-mar = Mar
month-apr = Apr
month-may = May
month-jun = Jun
month-jul = Jul
month-aug = Aug
month-sep = Sep
month-oct = Oct
month-nov = Nov
month-dec = Dec

## Store form
store-form-new-heading = New Store
store-form-edit-heading = Edit Store
store-form-name = Name
store-form-location = Location
store-form-opening-hours = Opening hours
store-form-opening-hours-hint = Pick "Closed" for days the store is shut.
store-form-define-opening-hours = Define opening hours
store-form-opens = Opens
store-form-closes = Closes
store-form-is-company = This store is the company
store-form-company = Company
store-form-company-choose = — choose —
store-form-company-description = Company description (optional)
store-form-company-homepage = Company homepage (optional)
store-form-product-heading = Products (at least one required)
store-form-product-n = Product

## Company form
company-form-heading = Edit company
company-form-name = Name
company-form-description = Description
company-form-homepage = Homepage

## Product form (add to store)
product-form-add-heading = Add product to "{ $name }"
product-form-new-checkbox = Create new product
product-form-product = Product
product-form-choose = — choose —
product-form-name = Product name
product-form-category = Category
product-form-description-optional = Description (optional)
product-form-seasonal-checkbox = Only available seasonally
product-form-seasonal-hint = Available all year by default — turn this on to pick specific months.

## Store-product (seasonality) form
store-product-seasonality-form-heading = Edit seasonality: { $name }

## Edit product form
edit-product-form-heading = Edit product
edit-product-form-name = Name
edit-product-form-category = Category
edit-product-form-description = Description

## Image form
image-form-heading = Add image
image-form-file-label = Image file (JPEG, PNG or WebP, max. 15 MB)
image-form-description-optional = Description (optional)
image-form-upload = Upload
image-form-alt-fallback = Product image

## Store detail — action labels
detail-edit-company = Edit company
detail-edit-store = Edit store
detail-delete-store = Delete store
detail-edit-product = Edit product (name, category, description)
detail-edit-product-title = Edit product
detail-edit-seasonality = Edit seasonality
detail-edit-seasonality-title = Edit seasonality
detail-remove-offer = Remove this offer from this store
detail-remove-offer-title = Remove offer

## Confirmation
confirmation-pending = Thanks! Your submission ("{ $name }") is being reviewed.
confirmation-updated = "{ $name }" was updated.
confirmation-image-pending = Thanks! Your image is being reviewed.

## Map
map-sidebar-collapse = Hide sidebar
map-sidebar-expand = Show sidebar
map-sidebar-width = Change sidebar width

## Language
language-de = German
language-en = English

## Offline (PWA fallback page)
offline-title = No connection
offline-body = BauernKarte needs an internet connection to load stores and products. Everything works as usual again once you're back online.
offline-retry = Try again

## Navbar — collapsed menu
nav-menu-toggle = Open or close menu

## Credential checklists (register + account forms)
policy-rule-met = met
policy-rule-unmet = not met yet
email-rule-valid = Valid email address
password-rule-length = At least 12 characters
password-rule-not-common = Not a commonly used password
password-rule-not-personal = Doesn't contain your name or email address

## Password policy — server-side rejection messages
password-rule-length-error = The password must be at least 12 characters long.
password-rule-too-long-error = The password must be at most 128 characters long.
password-rule-not-common-error = That password is used too commonly. Please choose a different one.
password-rule-not-personal-error = The password must not contain your name or email address.

## Admin area
admin-title = Admin area
admin-nav-label = Admin sections
admin-back-to-map = Back to the map
admin-nav-users = Users
admin-nav-companies = Companies
admin-nav-stores = Stores
admin-nav-products = Products
admin-nav-offers = Offers
admin-nav-images = Images

admin-blurb-companies = The business behind a store. One company can run several stores, which is why it is reviewed separately.
admin-blurb-stores = The place itself, with its position and opening hours. New entries stay invisible to the public until approved here.
admin-blurb-products = The shared catalog. A product exists once and is offered by any number of stores.
admin-blurb-offers = The link "this store sells this product", with its season. A new offer does not create a new product.
admin-blurb-images = Uploaded product photos — the one thing visitors see exactly as submitted.

admin-tab-pending = Pending
admin-tab-changes = Changes
admin-tab-deleted = Deleted
admin-queue-empty = Nothing to do here right now.

admin-pill-new = New
admin-pill-deleted = Deleted
admin-pill-edited = Edited

admin-action-view = View
admin-action-approve = Approve
admin-action-reject = Reject
admin-action-restore = Restore
admin-action-revert = Revert

admin-error-name-taken = That name is in use again. Rename the existing entry and try once more.

## Admin area — users
admin-users-heading = Users
admin-users-blurb = Create accounts, grant admin rights, and remove accounts.
admin-users-new = Create user
admin-users-create = Create
admin-users-is-admin = Grant admin rights
admin-users-role = Role
admin-users-contributions = Contributions
admin-users-registered = Registered
admin-users-actions = Actions
admin-users-admin = Admin
admin-users-member = User
admin-users-grant-admin = Make admin
admin-users-revoke-admin = Revoke admin
admin-users-delete = Delete
admin-users-delete-confirm = Yes, delete
admin-users-delete-warning =
    { $count ->
        [0] This account has no entries. Deleting cannot be undone.
        [one] One entry by this account stays in the catalog but loses its author. Deleting cannot be undone.
       *[other] { $count } entries by this account stay in the catalog but lose their author. Deleting cannot be undone.
    }
admin-users-created = User created.
admin-users-role-changed = Role changed.
admin-users-deleted = User deleted.
admin-users-error-invalid = The name or email address is missing or invalid.
admin-users-error-exists = That email address is already registered.
admin-users-error-password = The password does not meet the policy.
admin-users-error-self = You cannot change your own account here.
admin-users-error-seed = The provisioned admin account is the way back into the system and stays as it is.
admin-users-error-last-admin = That is the last admin — nobody could reach this area afterwards.

## Impressum
impressum-link = Legal notice
impressum-heading = Legal notice
impressum-unconfigured = The legal notice hasn't been filled in yet. An admin can complete it under "Site info".
impressum-operator = Operator
impressum-contact = Contact
impressum-phone = Phone
impressum-legal = Legal details
impressum-vat-id = VAT ID
impressum-register = Company register number
impressum-responsible = Responsible for the content
impressum-purpose = Editorial purpose
impressum-data-heading = Map data
impressum-data-osm = The map and part of the store data come from OpenStreetMap and its contributors, published under the Open Database License (ODbL).

## Admin area — site info
admin-nav-site-info = Site info
admin-site-info-blurb = The contact details shown on the legal notice page. Empty fields are left off it.
admin-site-info-operator = Operator name
admin-site-info-street = Street and number
admin-site-info-postal = Postcode
admin-site-info-city = City
admin-site-info-country = Country
admin-site-info-optional = Everything here is optional — a private operator has no VAT ID and no register entry.
admin-site-info-purpose = What this website is for
admin-site-info-saved = Site info saved.
