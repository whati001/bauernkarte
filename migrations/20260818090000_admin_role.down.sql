DELETE FROM "user" WHERE email = 'bauernkarte@rehka.dev';
DROP INDEX IF EXISTS user_admin_idx;
ALTER TABLE "user" DROP COLUMN admin;
