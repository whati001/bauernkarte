# Admin runbook

Moderation now has a UI: sign in as an admin and use the helmet in the
navbar, or go to `/admin` directly. It covers everything this file used
to describe as hand-run SQL — approving, rejecting, reverting an edit and
restoring a deleted row — for all five moderated tables, plus account
management.

The SQL is kept below for the cases the UI deliberately doesn't cover,
and for when the app itself won't start.

## The admin account

`bauernkarte@rehka.dev` is created by a migration with no usable
password. The first server start that finds it that way sets the password
from `ADMIN_PASSWORD` in `.env`, and never touches it again — so a
password changed later through **Konto** survives restarts even with
`ADMIN_PASSWORD` still set. A migration can't do this itself: it is
static SQL with no access to `.env` and no way to run Argon2, and a hash
committed to the repository would be a published password.

To reset a forgotten admin password, clear the hash and restart:

```sql
update "user" set pwd_hash = '' where email = 'bauernkarte@rehka.dev';
```

Grant admin rights to someone else in the UI (**Benutzer → Zum Admin
machen**), or directly:

```sql
update "user" set admin = true where email = '<address>';
```

The UI refuses to demote or delete your own account, and refuses to touch
`bauernkarte@rehka.dev` at all, so there is always a way back in.

## What the UI does

| Action | Effect |
|---|---|
| **Freigeben** | `approved = true` — the entry becomes publicly visible |
| **Ablehnen** | `deleted = true`. A *soft* delete, so it is still in **Gelöscht** and can be restored |
| **Zurücknehmen** | Re-applies an `edit_log` entry's `old_value` through the normal edit path, and logs the revert itself — a revert can be reverted |
| **Wiederherstellen** | `deleted = false` |

Every one of these writes an `edit_log` row, so "who approved this" has an
answer.

Rejecting is a soft delete rather than the `delete from …` this file used
to recommend, for two reasons. The catalog's foreign keys are all
`NO ACTION`, so hard-deleting a store that already has an offer fails
outright:

```
ERROR: update or delete on table "store" violates foreign key constraint
       "store_product_store_fkey" on table "store_product"
```

And a rejected row that still exists can be un-rejected. Nothing the
admin UI does is irreversible, with one exception: deleting a *user*.

## Deleting a user

`created_by`/`modified_by` are `ON DELETE SET NULL`, so an account's
submissions stay in the catalog and only lose their author. That cannot
be undone, which is why the UI shows the number of affected entries and
asks for a second click.

## Purging for real

The UI never hard-deletes catalog rows. To actually remove spam, delete
children first — the foreign keys are `NO ACTION`:

```sql
delete from image         where store_product in (select id from store_product where store = <id>);
delete from store_product where store = <id>;
delete from store         where id = <id> and not approved;
```

## Restoring a product whose name was taken

`product_name_key` only covers non-deleted rows, so a deleted product's
name is free to be reused. If it has been, restoring collides and the UI
says so rather than failing. Rename the live row first:

```sql
update product set name = '<new name>' where id = <live id>;
```

## Session cleanup

Handled automatically by a background task in the running server
(`continuously_delete_expired`, hourly) — no manual step needed.
