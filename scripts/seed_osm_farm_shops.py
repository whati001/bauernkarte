#!/usr/bin/env python3
"""Generate a SQL seed file from OpenStreetMap `shop=farm` data in Austria.

Source: Overpass API, tag documented at
https://wiki.openstreetmap.org/wiki/DE:Tag:shop%3Dfarm — farm shops selling
agricultural products either at the farm itself or as a roadside stand.

Per-shop `name` becomes both the `company.name` and `store.name` (per
request — this dataset doesn't distinguish farm-shop-as-a-business from
farm-shop-as-a-location, so the two are set identically, same as the
in-app "this store is the company" flow). Coordinates become
`store.position`. Where OSM's `produce=`/`product=`/`vending=` tags name
what's sold, those are mapped to this app's product catalog and a
`store_product` row is created — there's no price data in OSM, so prices
below are illustrative placeholders, not observed prices (documented
inline; do not treat as real).

Usage:
    python3 scripts/seed_osm_farm_shops.py > /tmp/osm_seed.sql
    psql "$DATABASE_URL" -f /tmp/osm_seed.sql

Everything is inserted `approved = true, created_by = NULL` — this is
curated reference data being seeded directly, not a simulated user
submission subject to moderation (same treatment as the `category` seed).
"""

import json
import sys
import urllib.request

OVERPASS_URL = "https://overpass-api.de/api/interpreter"
OVERPASS_QUERY = """
[out:json][timeout:120];
area["ISO3166-1"="AT"][admin_level=2]->.at;
(
  node["shop"="farm"](area.at);
  way["shop"="farm"](area.at);
);
out center tags;
"""

# OSM produce=/product=/vending= tokens (English, semicolon/comma
# separated in practice) -> (German product name, category name, EUR
# placeholder price). Tokens with no reasonable single-category home, or
# too vague to be a real product ("food", "groceries", "deli"), are left
# unmapped and simply skipped rather than guessed at.
PRODUCT_MAP = {
    "apple": ("Äpfel", "Obst & Gemüse", "2.50"),
    "apples": ("Äpfel", "Obst & Gemüse", "2.50"),
    "vegetables": ("Gemüse", "Obst & Gemüse", "2.80"),
    "vegetable": ("Gemüse", "Obst & Gemüse", "2.80"),
    "fruits": ("Obst", "Obst & Gemüse", "3.00"),
    "fruit": ("Obst", "Obst & Gemüse", "3.00"),
    "potatoes": ("Kartoffeln", "Obst & Gemüse", "1.80"),
    "strawberry": ("Erdbeeren", "Obst & Gemüse", "4.50"),
    "cherry": ("Kirschen", "Obst & Gemüse", "6.00"),
    "pumpkin_seed": ("Kürbiskerne", "Obst & Gemüse", "12.00"),
    "pumpkin_seeds": ("Kürbiskerne", "Obst & Gemüse", "12.00"),
    "herbs": ("Kräuter", "Obst & Gemüse", "2.00"),
    "egg": ("Eier", "Eier", "3.50"),
    "eggs": ("Eier", "Eier", "3.50"),
    "cheese": ("Käse", "Milchprodukte & Käse", "16.00"),
    "milk": ("Milch", "Milchprodukte & Käse", "1.30"),
    "yoghurt": ("Joghurt", "Milchprodukte & Käse", "2.20"),
    "jughurt": ("Joghurt", "Milchprodukte & Käse", "2.20"),
    "butter": ("Butter", "Milchprodukte & Käse", "4.50"),
    "buttermilk": ("Buttermilch", "Milchprodukte & Käse", "1.50"),
    "curd chease": ("Topfen", "Milchprodukte & Käse", "3.80"),
    "dairy": ("Milchprodukte", "Milchprodukte & Käse", "4.00"),
    "meat": ("Fleisch", "Fleisch & Wurst", "18.00"),
    "beef": ("Rindfleisch", "Fleisch & Wurst", "22.00"),
    "pork": ("Schweinefleisch", "Fleisch & Wurst", "15.00"),
    "bacon": ("Speck", "Fleisch & Wurst", "14.00"),
    "sausage": ("Wurst", "Fleisch & Wurst", "12.00"),
    "ham": ("Schinken", "Fleisch & Wurst", "19.00"),
    "chicken": ("Hühnerfleisch", "Fleisch & Wurst", "13.00"),
    "chicken_meat": ("Hühnerfleisch", "Fleisch & Wurst", "13.00"),
    "bread": ("Brot", "Brot & Backwaren", "4.50"),
    "bakery_products": ("Backwaren", "Brot & Backwaren", "5.00"),
    "honey": ("Honig", "Honig", "8.50"),
    "wine": ("Wein", "Getränke", "8.00"),
    "juice": ("Saft", "Getränke", "3.50"),
    "juices": ("Saft", "Getränke", "3.50"),
    "syrup": ("Sirup", "Getränke", "6.00"),
    "sirup": ("Sirup", "Getränke", "6.00"),
    "tea": ("Tee", "Getränke", "4.00"),
    "liquer": ("Likör", "Getränke", "15.00"),
    "liqueur": ("Likör", "Getränke", "15.00"),
    "noodles": ("Nudeln", "Sonstiges", "3.50"),
    "cereal": ("Getreide", "Sonstiges", "3.00"),
    "cereals": ("Getreide", "Sonstiges", "3.00"),
    "spices": ("Gewürze", "Sonstiges", "5.00"),
    "algae": ("Algen", "Sonstiges", "9.00"),
    "flowers": ("Blumen", "Sonstiges", "6.00"),
    "christmas tree": ("Christbaum", "Sonstiges", "25.00"),
    "soups": ("Suppen", "Sonstiges", "4.00"),
    "jam": ("Marmelade", "Sonstiges", "4.50"),
    "pumpkin_seed_oil": ("Kürbiskernöl", "Sonstiges", "11.00"),
    "fish": ("Fisch", "Sonstiges", "20.00"),
}


def sql_str(value):
    if value is None:
        return "NULL"
    escaped = value.replace("'", "''")
    return f"'{escaped}'"


def fetch_overpass():
    req = urllib.request.Request(
        OVERPASS_URL, data=("data=" + urllib.parse.quote(OVERPASS_QUERY)).encode()
    )
    with urllib.request.urlopen(req, timeout=120) as resp:
        return json.load(resp)


def extract_latlon(element):
    if "lat" in element and "lon" in element:
        return element["lat"], element["lon"]
    center = element.get("center")
    if center:
        return center["lat"], center["lon"]
    return None, None


def parse_products(tags):
    """Collect distinct (German name, category, price) tuples named by
    produce=/product=/vending= on one shop, splitting each on the mix of
    ';'/',' separators used in practice."""
    found = {}
    for key in ("produce", "product", "vending"):
        raw = tags.get(key)
        if not raw:
            continue
        for token in raw.replace(",", ";").split(";"):
            token = token.strip().lower()
            mapped = PRODUCT_MAP.get(token)
            if mapped:
                found[mapped[0]] = mapped
    return list(found.values())


def main():
    data = json.load(sys.stdin) if not sys.stdin.isatty() else fetch_overpass()
    elements = data["elements"]

    named = []
    for el in elements:
        tags = el.get("tags", {})
        name = tags.get("name")
        lat, lon = extract_latlon(el)
        if not name or lat is None or lon is None:
            continue
        named.append((el, name, lat, lon, tags))

    print(f"-- Generated from {len(named)} named OSM shop=farm elements in Austria")
    print(f"-- (of {len(elements)} total; the rest had no name tag or no coordinates)")
    print("-- See scripts/seed_osm_farm_shops.py for provenance and the product/price mapping.")
    print()

    # Product catalog: one INSERT covering every distinct product any shop
    # references, deduplicated up front, `ON CONFLICT (name) DO NOTHING` so
    # re-running this script is safe against an already-seeded product
    # table (e.g. "Äpfel" from manual testing).
    all_products = {}
    for _, _, _, _, tags in named:
        for name, category, price in parse_products(tags):
            all_products[name] = (category, price)

    if all_products:
        print("-- Product catalog referenced by the shops below.")
        print("INSERT INTO product (category, name, approved, created_by) VALUES")
        rows = []
        for name, (category, _price) in sorted(all_products.items()):
            rows.append(
                f"  ((SELECT id FROM category WHERE name = {sql_str(category)}), "
                f"{sql_str(name)}, true, NULL)"
            )
        print(",\n".join(rows) + "\nON CONFLICT (name) DO NOTHING;")
        print()

    print("-- Companies + stores, one pair per named shop=farm location.")
    print("-- Company/store id correspondence relies on sequential identity")
    print("-- assignment on an otherwise-untouched insert order below — run")
    print("-- this against a table with no concurrent writes.")
    print()

    for el, name, lat, lon, tags in named:
        website = tags.get("website") or tags.get("contact:website")
        opening_hours = tags.get("opening_hours")
        print("WITH new_company AS (")
        print(
            f"  INSERT INTO company (name, homepage, approved, created_by) "
            f"VALUES ({sql_str(name)}, {sql_str(website)}, true, NULL) RETURNING id"
        )
        print("), new_store AS (")
        print(
            "  INSERT INTO store (company, name, position, openinghours, approved, created_by) "
            f"SELECT id, {sql_str(name)}, "
            f"ST_SetSRID(ST_MakePoint({lon}, {lat}), 4326)::geography, "
            f"{sql_str(opening_hours)}, true, NULL FROM new_company RETURNING id"
        )
        print(")")
        products = parse_products(tags)
        if products:
            selects = []
            for pname, _category, price in products:
                # Explicit casts: a UNION ALL of several SELECTs each
                # carrying a bare `NULL` infers the merged column as
                # `text` instead of the target `bigint`, which Postgres
                # then refuses to insert — harmless with a single SELECT,
                # but every one of these has >=1 UNION ALL sibling.
                selects.append(
                    f"SELECT new_store.id, product.id, {price}::numeric, true, NULL::bigint "
                    f"FROM new_store, product WHERE product.name = {sql_str(pname)}"
                )
            print(
                "INSERT INTO store_product (store, product, price, approved, created_by)\n"
                + "\nUNION ALL\n".join(selects)
                + ";"
            )
        else:
            print("SELECT 1 FROM new_store;")
        print()

    print(f"-- Done: {len(named)} companies, {len(named)} stores, "
          f"{len(all_products)} distinct products.", file=sys.stderr)


if __name__ == "__main__":
    import urllib.parse
    main()
