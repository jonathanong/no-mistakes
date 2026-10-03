-- Synthetic schema for postgres-bounded-statements against a generated catalog.
CREATE SCHEMA bounded_demo;
SET search_path = bounded_demo, pg_catalog;

CREATE TABLE accounts (id uuid PRIMARY KEY, email text NOT NULL UNIQUE, name text);
CREATE TABLE orders (
  id uuid PRIMARY KEY,
  account_id uuid NOT NULL REFERENCES accounts (id),
  status text NOT NULL,
  slug text
);
-- A partial unique index covers only some rows, so it proves no uniqueness.
CREATE UNIQUE INDEX orders_slug_live_idx ON orders (slug) WHERE status <> 'void';
CREATE TABLE invoices (id uuid PRIMARY KEY, paid_at timestamptz);
CREATE TABLE exports (id uuid PRIMARY KEY, expires_at timestamptz NOT NULL, s3_key text);
CREATE TABLE currencies (code text PRIMARY KEY);
-- A deferrable constraint may hold duplicates inside a transaction.
CREATE TABLE tokens (token text UNIQUE DEFERRABLE INITIALLY DEFERRED);
