#!/usr/bin/env python3
"""Generate a SQL seed file from OpenStreetMap farm-produce data in Austria.

Source: Overpass API, two tags:

  * `shop=farm` — https://wiki.openstreetmap.org/wiki/DE:Tag:shop%3Dfarm —
    farm shops selling agricultural products either at the farm itself or
    as a roadside stand.
  * `amenity=vending_machine` with a farm-produce `vending=` value
    (see VENDING_TOKENS) — https://wiki.openstreetmap.org/wiki/DE:Tag:amenity%3Dvending_machine
    — the self-service "Regiomat"/milk-and-egg machines that sell the same
    goods around the clock, and belong on the same map.

Per-shop `name` becomes `store.name` and the coordinates become
`store.position`. OSM's `website=`/`contact:website=` tag is dropped —
there is nowhere to put it, the app has no store-level homepage field.
Where OSM's `produce=`/`product=`/`vending=` tags name
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

    # Or replay a previously saved Overpass response instead of hitting
    # the API again:
    python3 scripts/seed_osm_farm_shops.py < /tmp/osm_response.json > /tmp/osm_seed.sql

By default this picks fetch-vs-replay from whether stdin is a tty, which
only works when run directly in an interactive terminal. `--live` forces
a fresh Overpass fetch regardless of stdin — needed for any programmatic
caller (bootstrap.py's `stores` subcommand) whose own stdin isn't a real
tty either.

Everything is inserted `approved = true, created_by = NULL` — this is
curated reference data being seeded directly, not a simulated user
submission subject to moderation.

The generated SQL is idempotent: products upsert on their name (filling
in a missing icon and changing nothing else), and each shop's
store/store_product insert is guarded
on "no store of this name within 1m of this point" already existing. An
earlier version guarded only the products, so a second run silently
duplicated every shop — see the guard's own comment below for why that
was worse than merely redundant rows.
"""

import argparse
import json
import sys
import urllib.request

OVERPASS_URL = "https://overpass-api.de/api/interpreter"

# `vending=` values that mean "this machine sells farm produce". Every
# one of them has a PRODUCT_MAP entry below — a token here without one
# would pull the machines in and then drop them all again on the
# no-mappable-product filter in main().
VENDING_TOKENS = (
    "milk", "eggs", "tomatoes", "cheese", "sausages", "potatoes",
    "noodles", "meat", "honey", "fruits",
)

# Matched unanchored against the whole `vending` value, because it is
# routinely a multi-value list ("milk;eggs", "cheese;sausages;bread")
# that an anchored match would miss entirely. The looseness costs
# nothing: it also matches values that merely *contain* a token, but
# parse_products splits the value and looks each token up exactly, so a
# machine with no real farm-produce token maps to no product and is
# skipped.
OVERPASS_QUERY = """
[out:json][timeout:180];
area["ISO3166-1"="AT"][admin_level=2]->.at;
(
  node["shop"="farm"](area.at);
  way["shop"="farm"](area.at);
  node["amenity"="vending_machine"]["vending"~"%(vending)s"](area.at);
  way["amenity"="vending_machine"]["vending"~"%(vending)s"](area.at);
);
out center tags;
""" % {"vending": "|".join(VENDING_TOKENS)}

# OSM produce=/product=/vending= tokens (English, semicolon/comma
# separated in practice) -> German product name. Tokens too vague to be
# a real product ("food", "groceries", "deli") are left unmapped and
# simply skipped rather than guessed at. That holds for `vending=food`
# machines too, even though it is the most common value on them: a
# catch-all "assorted groceries" product is out of scope here, so those
# machines are neither queried for nor seeded.
PRODUCT_MAP = {
    "apple": "Äpfel",
    "apples": "Äpfel",
    "vegetables": "Gemüse",
    "vegetable": "Gemüse",
    "fruits": "Obst",
    "fruit": "Obst",
    "potatoes": "Kartoffeln",
    "tomatoes": "Tomaten",
    "strawberry": "Erdbeeren",
    "cherry": "Kirschen",
    "pumpkin_seed": "Kürbiskerne",
    "pumpkin_seeds": "Kürbiskerne",
    "herbs": "Kräuter",
    "egg": "Eier",
    "eggs": "Eier",
    "cheese": "Käse",
    "milk": "Milch",
    "yoghurt": "Joghurt",
    "jughurt": "Joghurt",
    "butter": "Butter",
    "buttermilk": "Buttermilch",
    "curd chease": "Topfen",
    "dairy": "Milchprodukte",
    "meat": "Fleisch",
    "beef": "Rindfleisch",
    "pork": "Schweinefleisch",
    "bacon": "Speck",
    "sausage": "Wurst",
    "sausages": "Wurst",
    "ham": "Schinken",
    "chicken": "Hühnerfleisch",
    "chicken_meat": "Hühnerfleisch",
    "bread": "Brot",
    "bakery_products": "Backwaren",
    "honey": "Honig",
    "wine": "Wein",
    "juice": "Saft",
    "juices": "Saft",
    "syrup": "Sirup",
    "sirup": "Sirup",
    "tea": "Tee",
    "liquer": "Likör",
    "liqueur": "Likör",
    "noodles": "Nudeln",
    "cereal": "Getreide",
    "cereals": "Getreide",
    "spices": "Gewürze",
    "algae": "Algen",
    "flowers": "Blumen",
    "christmas tree": "Christbaum",
    "soups": "Suppen",
    "jam": "Marmelade",
    "pumpkin_seed_oil": "Kürbiskernöl",
    "fish": "Fisch",
}


# Emoji shown next to each product in the UI. Lives here because this
# script is what creates these rows: the `product_icon` migration seeded
# icons with a one-shot UPDATE, and every product below is inserted
# *after* migrations run, so those rows arrived with `icon` NULL and the
# whole catalog rendered as the neutral package fallback. The migration
# `product_icon_backfill` repairs databases already in that state; this
# table is what keeps new ones out of it.
#
# A name missing here is not an error — `icon` is nullable and the
# templates fall back to a package, which is the intended treatment for
# products submitted through the app.
PRODUCT_ICONS = {
    "Äpfel": "🍎",
    "Obst": "🍇",
    "Gemüse": "🥦",
    "Kartoffeln": "🥔",
    "Tomaten": "🍅",
    "Erdbeeren": "🍓",
    "Kirschen": "🍒",
    "Kräuter": "🌿",
    "Kürbiskerne": "🎃",
    "Kürbiskernöl": "🛢️",
    "Eier": "🥚",
    "Käse": "🧀",
    "Milch": "🥛",
    "Milchprodukte": "🥛",
    "Joghurt": "🥣",
    "Butter": "🧈",
    "Buttermilch": "🥛",
    "Topfen": "🥣",
    "Fleisch": "🥩",
    "Rindfleisch": "🐄",
    "Schweinefleisch": "🐷",
    "Speck": "🥓",
    "Wurst": "🌭",
    "Schinken": "🍖",
    "Hühnerfleisch": "🍗",
    "Brot": "🍞",
    "Backwaren": "🥐",
    "Honig": "🍯",
    "Wein": "🍷",
    "Saft": "🧃",
    "Sirup": "🧴",
    "Tee": "🍵",
    "Likör": "🥃",
    "Nudeln": "🍝",
    "Getreide": "🌾",
    "Gewürze": "🧂",
    "Algen": "🌊",
    "Blumen": "💐",
    "Christbaum": "🎄",
    "Suppen": "🍲",
    "Marmelade": "🫙",
    "Fisch": "🐟",
}


def sql_str(value):
    if value is None:
        return "NULL"
    escaped = value.replace("'", "''")
    return f"'{escaped}'"


def fetch_overpass():
    # Overpass rejects urllib's default `Python-urllib/x.y` User-Agent
    # outright (406 Not Acceptable) — confirmed live, and matches
    # Overpass's own documented ask for a descriptive UA identifying the
    # calling app/contact.
    req = urllib.request.Request(
        OVERPASS_URL,
        data=("data=" + urllib.parse.quote(OVERPASS_QUERY)).encode(),
        headers={"User-Agent": "bauernkarte-bootstrap/1.0 (andreaskarner@outlook.com)"},
    )
    with urllib.request.urlopen(req, timeout=180) as resp:
        return json.load(resp)


def extract_latlon(element):
    if "lat" in element and "lon" in element:
        return element["lat"], element["lon"]
    center = element.get("center")
    if center:
        return center["lat"], center["lon"]
    return None, None


def display_name(tags):
    """What to call this location, or None if nothing usable is tagged.

    Farm shops in OSM carry a `name` almost without exception; vending
    machines mostly don't — for those `operator` (the farm running the
    machine) is the only other tag holding a real name, and is what
    people would recognise it by anyway. Anything with neither is left
    out by the caller rather than given a placeholder: hundreds of
    unrelated machines all listed as "Automat" would be worse than
    absent."""
    return tags.get("name") or tags.get("operator")


def parse_products(tags):
    """Collect the distinct German product names named by
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
                found[mapped] = None
    return list(found)


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument(
        "--live", action="store_true",
        help="fetch fresh from Overpass regardless of stdin (see module docstring)",
    )
    args = parser.parse_args()

    data = fetch_overpass() if args.live or sys.stdin.isatty() else json.load(sys.stdin)
    elements = data["elements"]

    named = []
    skipped_no_product = 0
    for el in elements:
        tags = el.get("tags", {})
        name = display_name(tags)
        lat, lon = extract_latlon(el)
        if not name or lat is None or lon is None:
            continue
        # A store with zero store_product rows doesn't show up in the
        # app's map at all (search/results are product-driven) — it'd
        # just be a dead store nobody can ever find. Skip
        # shops whose produce=/product=/vending= tags didn't map to
        # anything in PRODUCT_MAP, rather than insert one anyway.
        if not parse_products(tags):
            skipped_no_product += 1
            continue
        named.append((el, name, lat, lon, tags))

    print(f"-- Generated from {len(named)} named OSM shop=farm / farm-produce")
    print(f"-- amenity=vending_machine elements in Austria")
    print(f"-- (of {len(elements)} total; the rest had no name or operator tag, no "
          f"coordinates, or no mappable product — {skipped_no_product} skipped for "
          "the latter)")
    print("-- See scripts/seed_osm_farm_shops.py for provenance and the product mapping.")
    print()

    # Product catalog: one INSERT covering every distinct product any shop
    # references, deduplicated up front, `ON CONFLICT (name) DO NOTHING` so
    # re-running this script is safe against an already-seeded product
    # table (e.g. "Äpfel" from manual testing).
    all_products = set()
    for _, _, _, _, tags in named:
        all_products.update(parse_products(tags))

    if all_products:
        print("-- Product catalog referenced by the shops below.")
        print("INSERT INTO product (name, icon, approved, created_by) VALUES")
        rows = []
        for name in sorted(all_products):
            rows.append(
                f"  ({sql_str(name)}, {sql_str(PRODUCT_ICONS.get(name))}, true, NULL)"
            )
        # `WHERE NOT deleted` is required, not decorative: `product_name_key`
        # is a *partial* unique index (so a deleted product's name can be
        # reused), and Postgres rejects an ON CONFLICT target that doesn't
        # match the index predicate with "there is no unique or exclusion
        # constraint matching the ON CONFLICT specification".
        #
        # DO UPDATE rather than DO NOTHING so re-running this repairs a
        # catalog seeded before icons existed. The guard keeps it to that:
        # a product that already has an icon is left alone, so a hand-set
        # one is never overwritten.
        print(",\n".join(rows))
        print("ON CONFLICT (name) WHERE NOT deleted DO UPDATE")
        print("  SET icon = EXCLUDED.icon")
        print("  WHERE product.icon IS NULL;")
        print()

    print("-- Stores, one per named shop/machine location.")
    print()

    for el, name, lat, lon, tags in named:
        point = f"ST_SetSRID(ST_MakePoint({lon}, {lat}), 4326)::geography"
        # The whole per-shop statement hangs off this guard: if a store
        # with this name already sits on this spot, the store INSERT
        # selects no row, so `new_store` is empty, so the store_product
        # INSERT below it is empty too. Without it a
        # second run duplicated every shop at identical coordinates —
        # and two pins on the exact same point cluster into a permanent
        # "2" badge that no amount of zooming can split, which makes
        # those stores unreachable from the map (map.js redrawMarkers).
        # 1 metre rather than exact equality so re-running can't be
        # defeated by float representation drift through the geography
        # column.
        print("WITH new_store AS (")
        print(
            f"  INSERT INTO store (name, position, approved, created_by)\n"
            f"  SELECT {sql_str(name)}, {point}, true, NULL\n"
            f"  WHERE NOT EXISTS (\n"
            f"    SELECT 1 FROM store s\n"
            f"    WHERE s.name = {sql_str(name)} AND ST_DWithin(s.position, {point}, 1)\n"
            f"  )\n"
            f"  RETURNING id"
        )
        print(")")
        # `named` is pre-filtered to shops with >=1 mapped product, so
        # this is never empty here.
        selects = []
        for pname in parse_products(tags):
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
        print()

    print(f"-- Done: {len(named)} stores, "
          f"{len(all_products)} distinct products.", file=sys.stderr)


if __name__ == "__main__":
    import urllib.parse
    main()
