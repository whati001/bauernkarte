# German (source language — the app's original hardcoded copy). Keys are
# grouped by where they appear, not alphabetical, to stay legible next to
# the templates that use them.

## Navbar
nav-brand = Was hat der Bauer
nav-login = Login
nav-logout = Logout
nav-account = Konto
nav-new-store = Neues Geschäft

## Search sidebar
search-category = Kategorie
search-product = Produkt
search-distance = Umkreis
search-all = Alle
search-results-count = { $count ->
    [one] { $count } Ergebnis
   *[other] { $count } Ergebnisse
}
search-no-results = Keine Ergebnisse in diesem Umkreis.
search-no-results-nearby = { $count ->
    [one] Keine Ergebnisse in diesem Umkreis — { $count } Angebot weiter entfernt gefunden.
   *[other] Keine Ergebnisse in diesem Umkreis — { $count } Angebote weiter entfernt gefunden.
}
search-more = mehr
search-location-unavailable = Standort nicht verfügbar
search-pick-on-map = 👆 Auf der Karte klicken, um den Standort zu wählen
search-location-picked = 📍 Position gewählt
search-use-my-location = Meinen Standort verwenden

## Common actions
action-save = Speichern
action-delete = Löschen
action-cancel = Abbrechen
action-edit = Bearbeiten
action-back = Zurück
action-back-to-search = Zurück zur Suche

## Auth
auth-login-heading = Login
auth-register-heading = Registrieren
auth-email = E-Mail
auth-password = Passwort
auth-password-hint = Passwort (mind. 8 Zeichen)
auth-name = Name
auth-no-account = Noch kein Konto?
auth-have-account = Schon registriert?
auth-welcome-back = Willkommen zurück, { $name }!
auth-register-success = Willkommen, { $name }! Deine Registrierung war erfolgreich.

## Account page
account-heading = Konto
account-change-password = Passwort ändern
account-current-password = Aktuelles Passwort
account-new-password = Neues Passwort (mind. 8 Zeichen)
account-pending-heading = Meine Einträge (in Prüfung)
account-pending-empty = Keine ausstehenden Einträge.
account-profile-saved = Deine Kontodaten wurden gespeichert.
account-password-changed = Dein Passwort wurde geändert.

## Store detail
detail-open-in-maps = In Google Maps öffnen
detail-add-product = Produkt hinzufügen
detail-add-image = Bild hinzufügen
detail-rate = Bewerten
detail-unrate = Bewertung entfernen
detail-no-products = Noch keine Produkte für dieses Geschäft.

## Store form
store-form-new-heading = Neues Geschäft
store-form-edit-heading = Geschäft bearbeiten
store-form-name = Name
store-form-location = Standort
store-form-opening-hours = Öffnungszeiten
store-form-is-company = Dieses Geschäft ist die Firma
store-form-company = Firma
store-form-company-choose = — wählen —
store-form-company-description = Firmenbeschreibung (optional)
store-form-company-homepage = Firmen-Homepage (optional)
store-form-product-heading = Erstes Produkt (mindestens eines erforderlich)

## Company form
company-form-heading = Firma bearbeiten
company-form-name = Name
company-form-description = Beschreibung
company-form-homepage = Homepage

## Product form (add to store)
product-form-add-heading = Produkt zu "{ $name }" hinzufügen
product-form-new-checkbox = Neues Produkt anlegen
product-form-product = Produkt
product-form-choose = — wählen —
product-form-name = Produktname
product-form-category = Kategorie
product-form-description-optional = Beschreibung (optional)

## Edit product form
edit-product-form-heading = Produkt bearbeiten
edit-product-form-name = Name
edit-product-form-category = Kategorie
edit-product-form-description = Beschreibung

## Image form
image-form-heading = Bild hinzufügen
image-form-file-label = Bilddatei (JPEG, PNG oder WebP, max. 15 MB)
image-form-description-optional = Beschreibung (optional)
image-form-upload = Hochladen
image-form-alt-fallback = Produktbild

## Store detail — action labels
detail-edit-company = Firma bearbeiten
detail-edit-store = Geschäft bearbeiten
detail-delete-store = Geschäft löschen
detail-edit-product = Produkt bearbeiten (Name, Kategorie, Beschreibung)
detail-edit-product-title = Produkt bearbeiten
detail-remove-offer = Angebot bei diesem Geschäft entfernen
detail-remove-offer-title = Angebot entfernen

## Confirmation
confirmation-pending = Danke! Dein Eintrag ("{ $name }") wird geprüft.
confirmation-updated = "{ $name }" wurde aktualisiert.
confirmation-image-pending = Danke! Dein Bild wird geprüft.

## Map
map-sidebar-collapse = Seitenleiste ausblenden
map-sidebar-expand = Seitenleiste einblenden

## Language
language-de = Deutsch
language-en = Englisch
