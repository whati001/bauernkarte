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
`store_product` row is created.

`store.openinghours` and `store_product.seasonal_months` are left NULL:
OSM's `opening_hours` tag uses its own mini-language (e.g.
"Mo-Fr 08:00-18:00; Sa 08:00-12:00") that doesn't map onto this app's
per-weekday structured format (`src/opening_hours.rs`), and OSM has no
seasonality data for these shops at all — both are left for whoever
claims/edits the listing in-app rather than guessed at here.

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
# separated in practice) -> (German product name, category name).
# Tokens with no reasonable single-category home, or too vague to be a
# real product ("food", "groceries", "deli"), are left unmapped and
# simply skipped rather than guessed at.
PRODUCT_MAP = {
    "apple": ("Äpfel", "Obst & Gemüse"),
    "apples": ("Äpfel", "Obst & Gemüse"),
    "vegetables": ("Gemüse", "Obst & Gemüse"),
    "vegetable": ("Gemüse", "Obst & Gemüse"),
    "fruits": ("Obst", "Obst & Gemüse"),
    "fruit": ("Obst", "Obst & Gemüse"),
    "potatoes": ("Kartoffeln", "Obst & Gemüse"),
    "strawberry": ("Erdbeeren", "Obst & Gemüse"),
    "cherry": ("Kirschen", "Obst & Gemüse"),
    "pumpkin_seed": ("Kürbiskerne", "Obst & Gemüse"),
    "pumpkin_seeds": ("Kürbiskerne", "Obst & Gemüse"),
    "herbs": ("Kräuter", "Obst & Gemüse"),
    "egg": ("Eier", "Eier"),
    "eggs": ("Eier", "Eier"),
    "cheese": ("Käse", "Milchprodukte & Käse"),
    "milk": ("Milch", "Milchprodukte & Käse"),
    "yoghurt": ("Joghurt", "Milchprodukte & Käse"),
    "jughurt": ("Joghurt", "Milchprodukte & Käse"),
    "butter": ("Butter", "Milchprodukte & Käse"),
    "buttermilk": ("Buttermilch", "Milchprodukte & Käse"),
    "curd chease": ("Topfen", "Milchprodukte & Käse"),
    "dairy": ("Milchprodukte", "Milchprodukte & Käse"),
    "meat": ("Fleisch", "Fleisch & Wurst"),
    "beef": ("Rindfleisch", "Fleisch & Wurst"),
    "pork": ("Schweinefleisch", "Fleisch & Wurst"),
    "bacon": ("Speck", "Fleisch & Wurst"),
    "sausage": ("Wurst", "Fleisch & Wurst"),
    "ham": ("Schinken", "Fleisch & Wurst"),
    "chicken": ("Hühnerfleisch", "Fleisch & Wurst"),
    "chicken_meat": ("Hühnerfleisch", "Fleisch & Wurst"),
    "bread": ("Brot", "Brot & Backwaren"),
    "bakery_products": ("Backwaren", "Brot & Backwaren"),
    "honey": ("Honig", "Honig"),
    "wine": ("Wein", "Getränke"),
    "juice": ("Saft", "Getränke"),
    "juices": ("Saft", "Getränke"),
    "syrup": ("Sirup", "Getränke"),
    "sirup": ("Sirup", "Getränke"),
    "tea": ("Tee", "Getränke"),
    "liquer": ("Likör", "Getränke"),
    "liqueur": ("Likör", "Getränke"),
    "noodles": ("Nudeln", "Sonstiges"),
    "cereal": ("Getreide", "Sonstiges"),
    "cereals": ("Getreide", "Sonstiges"),
    "spices": ("Gewürze", "Sonstiges"),
    "algae": ("Algen", "Sonstiges"),
    "flowers": ("Blumen", "Sonstiges"),
    "christmas tree": ("Christbaum", "Sonstiges"),
    "soups": ("Suppen", "Sonstiges"),
    "jam": ("Marmelade", "Sonstiges"),
    "pumpkin_seed_oil": ("Kürbiskernöl", "Sonstiges"),
    "fish": ("Fisch", "Sonstiges"),
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
    """Collect distinct (German name, category) tuples named by
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
    print("-- See scripts/seed_osm_farm_shops.py for provenance and the product mapping.")
    print()

    # Product catalog: one INSERT covering every distinct product any shop
    # references, deduplicated up front, `ON CONFLICT (name) DO NOTHING` so
    # re-running this script is safe against an already-seeded product
    # table (e.g. "Äpfel" from manual testing).
    all_products = {}
    for _, _, _, _, tags in named:
        for name, category in parse_products(tags):
            all_products[name] = category

    if all_products:
        print("-- Product catalog referenced by the shops below.")
        print("INSERT INTO product (category, name, approved, created_by) VALUES")
        rows = []
        for name, category in sorted(all_products.items()):
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
        print("WITH new_company AS (")
        print(
            f"  INSERT INTO company (name, homepage, approved, created_by) "
            f"VALUES ({sql_str(name)}, {sql_str(website)}, true, NULL) RETURNING id"
        )
        print("), new_store AS (")
        print(
            "  INSERT INTO store (company, name, position, approved, created_by) "
            f"SELECT id, {sql_str(name)}, "
            f"ST_SetSRID(ST_MakePoint({lon}, {lat}), 4326)::geography, "
            "true, NULL FROM new_company RETURNING id"
        )
        print(")")
        products = parse_products(tags)
        if products:
            selects = []
            for pname, _category in products:
                # Explicit cast: a UNION ALL of several SELECTs each
                # carrying a bare `NULL` infers the merged column as
                # `text` instead of the target `bigint`, which Postgres
                # then refuses to insert — harmless with a single SELECT,
                # but every one of these has >=1 UNION ALL sibling.
                selects.append(
                    f"SELECT new_store.id, product.id, true, NULL::bigint "
                    f"FROM new_store, product WHERE product.name = {sql_str(pname)}"
                )
            print(
                "INSERT INTO store_product (store, product, approved, created_by)\n"
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
