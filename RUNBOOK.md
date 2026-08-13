# Admin runbook (v1 — no admin UI)

v1 has no in-app moderation interface. Every admin action below is a
direct SQL statement run against the Postgres database. This is a
deliberate design decision (design.md, content-moderation capability),
not a placeholder for a missing feature.

## Approve a pending submission

New `company`/`store`/`product`/`store_product`/`image` rows are created
with `approved = false` and are invisible to the public until approved:

```sql
update store set approved = true where id = <id>;
update company set approved = true where id = <id>;
update product set approved = true where id = <id>;
update store_product set approved = true where id = <id>;
update image set approved = true where id = <id>;
```

Find what's pending:

```sql
select id, name, created_by, created from store where not approved and not deleted order by created;
-- same shape for company, product; store_product/image need a join to be legible:
select sp.id, p.name as product, s.name as store, sp.price, sp.created_by, sp.created
from store_product sp join product p on p.id = sp.product join store s on s.id = sp.store
where not sp.approved and not sp.deleted order by sp.created;
```

## Reject a pending submission

There's no separate "rejected" state — rejecting just means it never
gets approved. To actually remove it (e.g. spam), delete it directly:

```sql
delete from store where id = <id> and not approved;
```

(Same pattern for the other four tables. Only ever delete rows that are
still `not approved` this way — an *approved* row should go through the
soft-delete path below instead, so it stays recoverable via `edit_log`.)

## Revert a bad edit

Every catalog edit (any logged-in user, any of the five entity types) is
logged to `edit_log` with the full row before and after:

```sql
select * from edit_log where entity_type = 'store' and entity_id = <id> order by changed desc;
```

`old_value` is the JSON snapshot from before the edit. To revert, apply
its fields back by hand, e.g. for a `store`:

```sql
update store
set name = 'Old Name', openinghours = 'Old Hours' -- etc, from old_value
where id = <id>;
```

There's no automated "restore this JSON blob" tool in v1 — this is a
manual, one-row-at-a-time operation, done deliberately given the current
scale.

## Restore a soft-deleted row

Deletes are soft (`deleted = true`), never destructive:

```sql
update store set deleted = false where id = <id>;
```

(Same for `company`, `product`, `store_product`, `image`.) Check
`edit_log` (`action = 'delete'`) first to see who deleted it and when.

## Session cleanup

Handled automatically by a background task in the running server
(`continuously_delete_expired`, hourly) — no manual step needed.
