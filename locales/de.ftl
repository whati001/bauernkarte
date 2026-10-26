# German (source language — the app's original hardcoded copy). Keys are
# grouped by where they appear, not alphabetical, to stay legible next to
# the templates that use them.

## Navbar
nav-brand = BauernKarte
nav-tagline = Lokal kaufen, Bauern stärken
nav-login = Login
nav-logout = Logout
nav-account = Konto
nav-account-of = Konto von { $name }
nav-new-store = Neues Geschäft
nav-search-label = Produkt suchen
nav-search-placeholder = Produkt…
nav-search-close = Suche schließen
nav-search-no-matches = Kein passendes Produkt
nav-search-clear = Suche zurücksetzen
nav-search-toggle = Suche öffnen oder schließen
nav-products-label = Beliebte Produkte
nav-products-all = Alle Produkte

## Search sidebar
search-filter-heading = Filter
search-product = Produkte
search-all = Alle
search-results-count = { $count ->
    [one] { $count } Ort in deiner Umgebung
   *[other] { $count } Orte in deiner Umgebung
}
search-no-results = Keine Geschäfte für diese Auswahl.
search-name = Nach Namen suchen
search-name-placeholder = Hofladen, Automat oder Markt suchen…
search-sort = Sortierung
search-sort-distance = Entfernung ↑
search-sort-name = Name A–Z
search-open-until = Heute bis { $name }
search-opens-at = Heute ab { $name }
search-closed-today = Heute geschlossen
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
auth-password-hint = Passwort (mind. 12 Zeichen)
auth-name = Name
auth-no-account = Noch kein Konto?
auth-have-account = Schon registriert?
auth-welcome-back = Willkommen zurück, { $name }!
auth-register-success = Willkommen, { $name }! Deine Registrierung war erfolgreich.

## Account page
account-heading = Konto
account-change-password = Passwort ändern
account-current-password = Aktuelles Passwort
account-new-password = Neues Passwort
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
detail-seasonal-availability = Saisonale Verfügbarkeit
detail-location = Standort
detail-rating-label = Bewertung
detail-photo-open = Foto öffnen
detail-photo-previous = Vorheriges Foto
detail-photo-next = Nächstes Foto
detail-photos = Fotos
detail-get-directions = Route berechnen
detail-store = Geschäft
detail-products = Produkte
detail-season = Saison
detail-product-count = { $count ->
    [one] { $count } Produkt
   *[other] { $count } Produkte
  }
detail-hero-art-alt = Illustration eines Bauernhofs mit den angebotenen Produkten
opening-hours-closed = Geschlossen

## Weekdays (opening-hours grid — store_form.html, sidebar_detail.html)
weekday-mon = Mo
weekday-tue = Di
weekday-wed = Mi
weekday-thu = Do
weekday-fri = Fr
weekday-sat = Sa
weekday-sun = So

## Months (seasonality grid — product_form.html, sidebar_detail.html)
month-jan = Jän
month-feb = Feb
month-mar = Mär
month-apr = Apr
month-may = Mai
month-jun = Jun
month-jul = Jul
month-aug = Aug
month-sep = Sep
month-oct = Okt
month-nov = Nov
month-dec = Dez

## Store form
store-form-new-heading = Neues Geschäft
store-form-edit-heading = Geschäft bearbeiten
store-form-name = Name
store-form-kind = Art
store-kind-market = Bauernmarkt
store-kind-vending-machine = Automat
store-kind-shop = Hofladen
store-form-location = Standort
store-form-opening-hours = Öffnungszeiten
store-form-opening-hours-hint = An geschlossenen Tagen "Geschlossen" auswählen.
store-form-define-opening-hours = Öffnungszeiten definieren
store-form-opens = Beginn
store-form-closes = Ende
store-form-product-heading = Produkte (mindestens eines erforderlich)
store-form-product-n = Produkt

## Product form (add to store)
product-form-add-heading = Produkt zu "{ $name }" hinzufügen
product-form-new-checkbox = Neues Produkt anlegen
product-form-product = Produkt
product-form-choose = — wählen —
product-form-name = Produktname
product-form-description-optional = Beschreibung (optional)
product-form-seasonal-checkbox = Nur saisonal verfügbar
product-form-seasonal-hint = Standardmäßig das ganze Jahr über verfügbar — nur aktivieren, um bestimmte Monate auszuwählen.

## Store-product (seasonality) form
store-product-seasonality-form-heading = Saisonalität bearbeiten: { $name }

## Edit product form
edit-product-form-heading = Produkt bearbeiten
edit-product-form-name = Name
edit-product-form-description = Beschreibung

## Image form
image-form-heading = Bild hinzufügen
image-form-file-label = Bilddatei (JPEG, PNG oder WebP, max. 15 MB)
image-form-description-optional = Beschreibung (optional)
image-form-upload = Hochladen
image-form-alt-fallback = Produktbild

## Store detail — action labels
detail-edit-store = Geschäft bearbeiten
detail-delete-store = Geschäft löschen
detail-edit-product = Produkt bearbeiten (Name, Beschreibung)
detail-edit-product-title = Produkt bearbeiten
detail-edit-seasonality = Saisonalität bearbeiten
detail-edit-seasonality-title = Saisonalität bearbeiten
detail-remove-offer = Angebot bei diesem Geschäft entfernen
detail-remove-offer-title = Angebot entfernen

## Confirmation
confirmation-pending = Danke! Dein Eintrag ("{ $name }") wird geprüft.
confirmation-updated = "{ $name }" wurde aktualisiert.
confirmation-image-pending = Danke! Dein Bild wird geprüft.

## Map
map-sidebar-collapse = Seitenleiste ausblenden
map-sidebar-expand = Seitenleiste einblenden
map-sidebar-width = Breite der Seitenleiste ändern

## Language
language-de = Deutsch
language-en = Englisch

## Offline (PWA fallback page)
offline-title = Keine Verbindung
offline-body = BauernKarte braucht eine Internetverbindung, um Geschäfte und Produkte zu laden. Sobald du wieder online bist, funktioniert alles wie gewohnt.
offline-retry = Erneut versuchen

## Navbar — collapsed menu
nav-menu-toggle = Menü öffnen oder schließen

## Credential checklists (register + account forms)
policy-rule-met = erfüllt
policy-rule-unmet = noch nicht erfüllt
email-rule-valid = Gültige E-Mail-Adresse
password-rule-length = Mindestens 12 Zeichen
password-rule-not-common = Kein häufig verwendetes Passwort
password-rule-not-personal = Enthält nicht deinen Namen oder deine E-Mail-Adresse

## Password policy — server-side rejection messages
password-rule-length-error = Das Passwort muss mindestens 12 Zeichen haben.
password-rule-too-long-error = Das Passwort darf höchstens 128 Zeichen haben.
password-rule-not-common-error = Dieses Passwort wird zu häufig verwendet. Bitte wähle ein anderes.
password-rule-not-personal-error = Das Passwort darf deinen Namen oder deine E-Mail-Adresse nicht enthalten.

## Adminbereich
admin-title = Adminbereich
admin-nav-label = Adminbereiche
admin-back-to-map = Zurück zur Karte
admin-nav-users = Benutzer
admin-nav-stores = Geschäfte
admin-nav-products = Produkte
admin-nav-offers = Angebote
admin-nav-images = Bilder

admin-blurb-stores = Der Verkaufsort selbst, mit Position und Öffnungszeiten. Neue Einträge sind öffentlich unsichtbar, bis sie hier freigegeben werden.
admin-blurb-products = Der gemeinsame Katalog. Ein Produkt existiert einmal und wird von beliebig vielen Geschäften angeboten.
admin-blurb-offers = Die Verbindung „dieses Geschäft führt dieses Produkt“, samt Saison. Ein neues Angebot legt kein neues Produkt an.
admin-blurb-images = Hochgeladene Produktbilder — das Einzige, was Besucher unverändert zu sehen bekommen.

admin-tab-pending = Offen
admin-tab-existing = Bestehend
admin-tab-changes = Änderungen
admin-tab-deleted = Gelöscht
admin-queue-empty = Hier ist gerade nichts zu tun.

admin-pill-new = Neu
admin-pill-deleted = Gelöscht
admin-pill-edited = Bearbeitet

admin-action-view = Ansehen
admin-action-approve = Freigeben
admin-action-reject = Ablehnen
admin-action-restore = Wiederherstellen
admin-action-revert = Zurücknehmen
admin-action-edit = Bearbeiten
admin-product-stores = { $count ->
    [one] von 1 Geschäft angeboten
   *[other] von { $count } Geschäften angeboten
}
admin-product-delete-warning = { $count ->
    [0] Das Produkt wird aus dem Katalog entfernt. Kein Geschäft bietet es an.
    [one] Das Produkt wird aus dem Katalog und aus dem 1 Geschäft entfernt, das es anbietet. Beides lässt sich unter „Gelöscht“ wiederherstellen.
   *[other] Das Produkt wird aus dem Katalog und aus den { $count } Geschäften entfernt, die es anbieten. Alles lässt sich unter „Gelöscht“ wiederherstellen.
}
admin-product-deleted = Produkt gelöscht.
admin-image-cover = Bild des Geschäfts
admin-image-owner = Porträt
admin-image-saved = Bild gespeichert.
admin-image-deleted = Bild gelöscht.
admin-image-delete-warning = Das Bild wird beim Geschäft entfernt und nicht mehr angezeigt. Es lässt sich unter „Gelöscht“ wiederherstellen.
admin-product-icon = Symbol
admin-product-icon-hint = Eines auswählen oder ein beliebiges Emoji eingeben bzw. einfügen. Leer zeigt 📦.
admin-product-error-icon = Das Symbol muss ein einzelnes Emoji sein.

admin-error-name-taken = Der Name ist inzwischen wieder vergeben. Benenne den bestehenden Eintrag um und versuche es erneut.

## Adminbereich — Benutzer
admin-users-heading = Benutzer
admin-users-blurb = Anlegen, Adminrechte vergeben und Konten entfernen.
admin-users-new = Benutzer anlegen
admin-users-create = Anlegen
admin-users-is-admin = Adminrechte geben
admin-users-role = Rolle
admin-users-contributions = Beiträge
admin-users-registered = Registriert
admin-users-actions = Aktionen
admin-users-admin = Admin
admin-users-member = Benutzer
admin-users-grant-admin = Zum Admin machen
admin-users-revoke-admin = Adminrechte entziehen
admin-users-delete = Löschen
admin-users-delete-confirm = Wirklich löschen
admin-users-delete-warning =
    { $count ->
        [0] Dieses Konto hat keine Einträge. Löschen lässt sich nicht rückgängig machen.
        [one] Ein Eintrag dieses Kontos bleibt bestehen, verliert aber den Urheber. Löschen lässt sich nicht rückgängig machen.
       *[other] { $count } Einträge dieses Kontos bleiben bestehen, verlieren aber den Urheber. Löschen lässt sich nicht rückgängig machen.
    }
admin-users-created = Benutzer angelegt.
admin-users-role-changed = Rolle geändert.
admin-users-deleted = Benutzer gelöscht.
admin-users-error-invalid = Name oder E-Mail-Adresse fehlt oder ist ungültig.
admin-users-error-exists = Diese E-Mail-Adresse ist bereits registriert.
admin-users-error-password = Das Passwort erfüllt die Richtlinie nicht.
admin-users-error-self = Das eigene Konto lässt sich hier nicht ändern.
admin-users-error-seed = Das eingerichtete Admin-Konto ist der Weg zurück ins System und bleibt unverändert.
admin-users-error-last-admin = Das ist der letzte Admin — sonst kommt niemand mehr in diesen Bereich.

## Impressum
impressum-link = Impressum
impressum-heading = Impressum
impressum-unconfigured = Das Impressum ist noch nicht hinterlegt. Ein Admin kann es unter „Seiteninfo" ausfüllen.
impressum-operator = Medieninhaber und Betreiber
impressum-contact = Kontakt
impressum-phone = Telefon
impressum-legal = Rechtliche Angaben
impressum-vat-id = UID-Nummer
impressum-register = Firmenbuchnummer
impressum-responsible = Verantwortlich für den Inhalt
impressum-purpose = Blattlinie
impressum-data-heading = Kartendaten
impressum-data-osm = Kartendarstellung und ein Teil der Geschäftsdaten stammen von OpenStreetMap und dessen Mitwirkenden, veröffentlicht unter der Open Database License (ODbL).

## Datenschutz
datenschutz-link = Datenschutz
datenschutz-heading = Datenschutz
datenschutz-intro = Diese Datenschutzerklärung beschreibt die Verarbeitung personenbezogener Daten beim Besuch und bei der Nutzung der BauernKarte.
datenschutz-updated = Stand: 26. Oktober 2026.
datenschutz-controller-heading = Verantwortlicher
datenschutz-controller-body = Verantwortlicher im Sinn des Art. 4 Z 7 DSGVO ist:
datenschutz-controller-unconfigured = Die Datenschutzangaben sind noch nicht hinterlegt. Ein Admin kann sie unter /admin/privacy-info ausfüllen.
datenschutz-controller-email = Datenschutz-E-Mail
datenschutz-controller-phone = Telefon
datenschutz-technical-heading = Technische Bereitstellung und Sicherheit
datenschutz-technical-body = Bei der HTTP-Nutzung werden technisch erforderliche Verbindungs- und Anfragedaten verarbeitet, darunter die IP-Adresse auf Transport- und Hosting-Ebene, Datum und Uhrzeit, angeforderte URL und Methode, Antwortstatus sowie vom Browser übermittelte Browser- und Geräteinformationen. Das dient der Auslieferung der Anwendung, Fehlerdiagnose und dem Schutz vor Missbrauch; verändernde Anfragen werden zusätzlich durch eine flüchtige, im Arbeitsspeicher geführte Ratenbegrenzung pro IP-Adresse geschützt. Rechtsgrundlage ist Art. 6 Abs. 1 lit. f DSGVO; berechtigtes Interesse ist der sichere und zuverlässige Betrieb.
datenschutz-account-heading = Konten und Sitzungen
datenschutz-account-body = Für Registrierung, Anmeldung, Profil, Berechtigungen und Gemeinschaftsfunktionen verarbeitet BauernKarte Name, E-Mail-Adresse, einen gesalzenen Argon2-Passwort-Hash, Rolle, Zeitstempel und die serverseitige Zuordnung der Sitzung. Rechtsgrundlage ist Art. 6 Abs. 1 lit. b DSGVO. Passwörter werden nicht im Klartext gespeichert.
datenschutz-content-heading = Gemeinschaftsinhalte und Moderation
datenschutz-content-body = Bei Einreichungen und Bearbeitungen werden Geschäftsposition, Adresse, Telefonnummer, Name und Biografie der betreibenden Person, Produktangaben, Bewertungen, Bildbeschreibungen, Geschäftsfotos und Porträts, Kennungen der erstellenden und bearbeitenden Konten, Zeitstempel sowie Prüf- und Änderungsschnappschüsse verarbeitet. Zwecke sind Moderation, Zuordnung, Betrieb des Verzeichnisses, Missbrauchsbehandlung und Wiederherstellung von Bearbeitungen. Rechtsgrundlagen sind Art. 6 Abs. 1 lit. b und lit. f DSGVO; berechtigte Interessen sind ein verlässliches, nachvollziehbares Gemeinschaftsverzeichnis und dessen Schutz. Importierte Geschäftsdaten können aus OpenStreetMap und von dessen Community-Mitwirkenden stammen; dies ist die Datenquelle im Sinn des Art. 14 DSGVO.
datenschutz-public-warning = Freigegebene Kontakt- und Betreiberangaben sowie freigegebene Bilder eines Geschäfts sind öffentlich. E-Mail-Adressen von Konten und Passwort-Hashes sind nicht öffentlich.
datenschutz-location-heading = Standort
datenschutz-location-body = Die Karte fragt derzeit beim Initialisieren nach dem Browserstandort. Erst nach Browserfreigabe werden Breiten- und Längengrad an den gleichursprünglichen Endpunkt /api/stores übermittelt, um Ergebnisse nach Entfernung zu sortieren. Die Koordinaten werden nicht in die Anwendungsdatenbank geschrieben, können aber als Abfrageparameter in Anfrage- oder Hosting-Protokollen vorkommen. Rechtsgrundlage ist Ihre Einwilligung nach Art. 6 Abs. 1 lit. a DSGVO. Bei Ablehnung bleibt die alphabetische Suche nutzbar. Die Berechtigung kann in den Browser- oder Website-Einstellungen widerrufen werden.
datenschutz-osm-heading = OpenStreetMap und externe Links
datenschutz-osm-body = Beim Laden der Karte ruft der Browser Rasterkarten direkt von tile.openstreetmap.org ab. Empfänger ist die OpenStreetMap Foundation (OSMF) im Vereinigten Königreich. Dabei können IP-Adresse, Browser, Gerät und Betriebssystem, Referrer, Datum und Uhrzeit sowie die angeforderten Kartenkacheln verarbeitet werden. Zweck ist die Kartendarstellung; Rechtsgrundlage ist Art. 6 Abs. 1 lit. f DSGVO und das berechtigte Interesse an einer nutzbaren geografischen Darstellung. OSMF betreibt dafür ein weltweites Cache-Netz.
datenschutz-osm-policy = Datenschutzerklärung der OpenStreetMap Foundation
datenschutz-google-body = Der gewöhnliche Link zur Routenplanung in Google Maps stellt erst dann eine Verbindung zu Google her, wenn Sie ihm folgen. Google verarbeitet die dabei anfallenden Daten unter eigenen Bedingungen.
datenschutz-storage-heading = Cookies, lokaler Speicher und Offline-Cache
datenschutz-storage-session = Notwendiges Server-Sitzungscookie: Ablauf nach 14 Tagen Inaktivität, SameSite=Lax und, wenn konfiguriert, Secure.
datenschutz-storage-locale = Cookie locale für die Sprachauswahl: höchstens ein Jahr, SameSite=Lax.
datenschutz-storage-sidebar = localStorage-Schlüssel bk-sidebar-width für die gewählte Seitenleistenbreite: bis die Website-Daten gelöscht werden.
datenschutz-storage-pwa = PWA-Cache für statische Dateien und die Offline-Seite: bis zur Ersetzung durch eine neue Version, zur Abmeldung des Service Workers oder zum Löschen der Website-Daten.
datenschutz-storage-osm-cache = Browser-Cache für OpenStreetMap-Kacheln: nach den Cache-Regeln des Browsers und des Kacheldienstes.
datenschutz-storage-legal = Diese Speicherungen sind funktional beziehungsweise technisch erforderlich und beruhen auf § 165 Abs. 3 TKG 2021. BauernKarte setzt keine Werbe-, Marketing- oder Analyse-Cookies ein.
datenschutz-recipients-heading = Empfänger und Übermittlungen
datenschutz-recipients-body = Empfänger können autorisierte Administratoren, die Kategorien der Hosting- und Datenbankbetreiber des Verantwortlichen, öffentliche Besucher bei freigegebenen öffentlichen Inhalten, die OpenStreetMap Foundation beim Kartenabruf sowie Behörden bei gesetzlicher Verpflichtung sein. OSMF hat ihren Sitz im Vereinigten Königreich und nutzt für Kartenkacheln ein weltweites Cache-Netz.
datenschutz-retention-heading = Speicherdauer
datenschutz-retention-body = Sitzungen verfallen nach 14 Tagen Inaktivität; abgelaufene Sitzungen werden stündlich bereinigt. Die Sprachauswahl bleibt höchstens ein Jahr gespeichert, die Seitenleistenbreite bis zum Löschen der Website-Daten und der Service-Worker-Cache bis zu Ersetzung, Abmeldung oder Löschung. Konten bleiben bis zur endgültigen Löschung durch einen Administrator auf Anfrage bestehen; erhaltene Beiträge verlieren danach ihre Kontozuordnung. Eigene Bewertungen können sofort entfernt werden. Beitrags- und Prüfdaten haben keine automatische Ablauffrist und bleiben erhalten, solange sie für Verzeichnisbetrieb, Moderation, Wiederherstellung, Rechtsansprüche oder eine begründete Löschentscheidung benötigt werden. Anfrage- und Sicherheitsprotokolle sowie Sicherungen folgen dem für Sicherheit und Wiederherstellung erforderlichen Zeitplan des Verantwortlichen beziehungsweise seines Hosting-Betreibers.
datenschutz-rights-heading = Ihre Rechte
datenschutz-right-information = Recht auf Information über die Verarbeitung.
datenschutz-right-access = Recht auf Auskunft über Ihre personenbezogenen Daten.
datenschutz-right-rectification = Recht auf Berichtigung unrichtiger Daten.
datenschutz-right-erasure = Recht auf Löschung, soweit die gesetzlichen Voraussetzungen vorliegen.
datenschutz-right-restriction = Recht auf Einschränkung der Verarbeitung.
datenschutz-right-portability = Recht auf Datenübertragbarkeit, soweit anwendbar.
datenschutz-right-objection = Recht auf Widerspruch gegen Verarbeitungen nach Art. 6 Abs. 1 lit. f DSGVO aus Gründen Ihrer besonderen Situation.
datenschutz-right-withdrawal = Recht, die Einwilligung zur Standortverarbeitung jederzeit ohne Wirkung auf die Rechtmäßigkeit der vorherigen Verarbeitung zu widerrufen.
datenschutz-rights-contact = Anfragen können an die oben unabhängig konfigurierte Datenschutz-E-Mail-Adresse gerichtet werden. Sie werden grundsätzlich innerhalb eines Monats beantwortet.
datenschutz-no-automated-decisions = BauernKarte führt kein Profiling und keine ausschließlich automatisierten Entscheidungen mit rechtlicher oder ähnlich erheblicher Wirkung durch.
datenschutz-complaint-heading = Beschwerderecht
datenschutz-complaint = Sie können sich bei einer Datenschutzaufsichtsbehörde beschweren, insbesondere bei der österreichischen Datenschutzbehörde:
datenschutz-dsb-name = Österreichische Datenschutzbehörde
datenschutz-dsb-address = Barichgasse 40–42, 1030 Wien
datenschutz-dsb-link = Kontaktseite der Datenschutzbehörde

## Adminbereich — Seiteninfo
admin-nav-site-info = Seiteninfo
admin-site-info-blurb = Die Kontaktdaten, die auf der Impressum-Seite stehen. Leere Felder werden dort weggelassen.
admin-site-info-operator = Name des Betreibers
admin-site-info-street = Straße und Hausnummer
admin-site-info-postal = PLZ
admin-site-info-city = Ort
admin-site-info-country = Land
admin-site-info-optional = Alles hier ist optional — ohne Gewerbe gibt es weder UID noch Firmenbucheintrag.
admin-site-info-purpose = Grundlegende Ausrichtung der Website
admin-site-info-saved = Seiteninfo gespeichert.

## Adminbereich — Datenschutz
admin-nav-privacy-info = Datenschutz
admin-privacy-info-blurb = Die unabhängig vom Impressum verwalteten Angaben zum datenschutzrechtlich Verantwortlichen. Alle Felder sind optional.
admin-privacy-info-controller-heading = Verantwortlicher
admin-privacy-info-controller = Name des Verantwortlichen
admin-privacy-info-street = Straße und Hausnummer
admin-privacy-info-postal = PLZ
admin-privacy-info-city = Ort
admin-privacy-info-country = Land
admin-privacy-info-contact-heading = Kontakt
admin-privacy-info-email = E-Mail
admin-privacy-info-phone = Telefon
admin-privacy-info-saved = Datenschutzangaben gespeichert.

## Store info (redesigned detail panel)
detail-close = Schließen
detail-shop-owner = Betrieben von
detail-owner-since = Am Hof seit { $year }
detail-call = Anrufen: { $name }
detail-reviews-count = { $count ->
    [one] ({ $count } Bewertung)
   *[other] ({ $count } Bewertungen)
  }
detail-no-reviews = Noch keine Bewertungen
detail-your-rating = Deine Bewertung
detail-rate-store = Hof bewerten
detail-remove-review = Bewertung entfernen
detail-stars = { $count ->
    [one] { $count } Stern
   *[other] { $count } Sterne
  }
detail-in-season-count = { $count } in Saison
detail-in-season = In Saison
detail-out-of-season = Außer Saison
detail-year-round = Ganzjährig
detail-hours-daily = Täglich
detail-hours-show-week = Ganze Woche
detail-more-actions = Mehr zu diesem Produkt
## Store form — new fields
store-form-contact-heading = Adresse und Kontakt
store-form-address = Adresse
store-form-phone = Telefon
store-form-owner-heading = Betreiber·in (optional)
store-form-owner-name = Name
store-form-owner-since = Am Hof seit (Jahr)
store-form-owner-bio = Kurzvorstellung
## Image form — owner portrait
image-form-is-cover = Als Bild des Geschäfts verwenden
image-form-is-cover-hint = Wird oben beim Geschäft und in der Suchliste gezeigt, sobald das Bild freigegeben ist.
image-form-is-owner = Das ist ein Porträt der Betreiberin / des Betreibers
## Validation
error-generic = Etwas ist schiefgelaufen. Bitte versuche es erneut.
error-login-required = Bitte zuerst einloggen.
error-name-required = Bitte einen Namen angeben.
error-email-invalid = Bitte eine gültige E-Mail-Adresse angeben.
error-email-taken = Diese E-Mail-Adresse ist bereits registriert.
error-login-failed = E-Mail oder Passwort ist falsch.
error-current-password-wrong = Aktuelles Passwort ist falsch.
error-admin-password-env = Das Passwort dieses Kontos wird über ADMIN_PASSWORD in .env festgelegt — dort ändern und die App neu starten.
error-product-required = Bitte ein Produkt wählen.
error-product-name-required = Bitte einen Produktnamen angeben.
error-product-name-taken = Ein Produkt mit diesem Namen gibt es schon — bitte aus der Liste wählen.
error-products-min-one = Bitte mindestens ein Produkt angeben.
error-season-month-required = Bitte mindestens einen Monat auswählen, oder „Nur saisonal verfügbar" deaktivieren.
error-hours-order = Öffnungszeiten: Beginn muss vor Ende liegen.
error-hours-incomplete = Öffnungszeiten: Bitte für jeden Tag entweder Beginn und Ende angeben oder beides leer lassen.
error-location-required = Bitte den Standort auf der Karte wählen.
error-image-required = Bitte eine Bilddatei auswählen.
error-image-too-large = Die Datei ist zu groß (max. 15 MB).
error-image-format = Nur JPEG, PNG oder WebP werden unterstützt.
error-image-decode = Bild konnte nicht gelesen werden (zu groß oder beschädigt).
error-owner-since-invalid = Bitte ein Jahr zwischen 1800 und heute angeben.
error-stars-range = Bitte 1 bis 5 Sterne vergeben.
error-not-found = Dieser Eintrag existiert nicht (mehr).
error-deleted = Dieser Eintrag wurde gelöscht.
