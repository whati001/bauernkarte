# English translation. Keys must match locales/de.ftl exactly — see
# src/i18n.rs's startup check, which fails loudly on drift instead of
# silently falling back key-by-key.

## Navbar
nav-brand = Was hat der Bauer
nav-login = Login
nav-logout = Logout
nav-account = Account
nav-new-store = New Store

## Search sidebar
search-category = Category
search-product = Product
search-distance = Radius
search-all = All
search-results-count = { $count ->
    [one] { $count } result
   *[other] { $count } results
}
search-no-results = No results in this radius.
search-no-results-nearby = { $count ->
    [one] No results in this radius — { $count } offer found further away.
   *[other] No results in this radius — { $count } offers found further away.
}
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
auth-password-hint = Password (min. 8 characters)
auth-name = Name
auth-no-account = No account yet?
auth-have-account = Already registered?
auth-welcome-back = Welcome back, { $name }!
auth-register-success = Welcome, { $name }! Your registration was successful.

## Account page
account-heading = Account
account-change-password = Change password
account-current-password = Current password
account-new-password = New password (min. 8 characters)
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
store-form-opening-hours-hint = Leave blank if closed that day.
store-form-is-company = This store is the company
store-form-company = Company
store-form-company-choose = — choose —
store-form-company-description = Company description (optional)
store-form-company-homepage = Company homepage (optional)
store-form-product-heading = First product (at least one required)

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

## Language
language-de = German
language-en = English
