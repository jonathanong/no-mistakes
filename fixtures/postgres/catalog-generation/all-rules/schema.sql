-- Synthetic schema for the generated catalog in schema.json. Requires PostgreSQL 18:
-- `VIRTUAL` generated columns and the deparse text in the catalog are version-specific.
-- Each object is built to trip one catalog rule enabled in .no-mistakes.yml.
CREATE SCHEMA shared;
CREATE TYPE shared.priority AS ENUM ('low', 'high');
CREATE SCHEMA partition_roots;

CREATE SCHEMA catalog_demo;
SET search_path = catalog_demo, pg_catalog;

CREATE TYPE invoice_state AS ENUM ('draft', 'sent', 'paid');

CREATE FUNCTION fn_touch_updated_at() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
  NEW.updated_at := now();
  RETURN NEW;
END
$$;

-- Two functions with the same body: postgres-duplicate-function-body.
CREATE FUNCTION fn_reject_account_delete() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN RAISE EXCEPTION 'rows are immutable' USING ERRCODE = '23514'; END
$$;
CREATE FUNCTION fn_reject_order_delete() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN RAISE EXCEPTION 'rows are immutable' USING ERRCODE = '23514'; END
$$;

CREATE TABLE accounts (
  id uuid PRIMARY KEY,
  email text NOT NULL,
  -- postgres-array-columns: an array column.
  tags text[] NOT NULL DEFAULT '{}',
  -- postgres-column-naming: a boolean column must read as a predicate.
  active boolean NOT NULL DEFAULT true,
  priority shared.priority NOT NULL DEFAULT 'low',
  updated_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT accounts_email_key UNIQUE (email)
);
COMMENT ON TABLE accounts IS 'People and organizations that own orders.';
CREATE TRIGGER trigger_accounts_touch BEFORE UPDATE ON accounts
  FOR EACH ROW EXECUTE FUNCTION fn_touch_updated_at();
CREATE TRIGGER trigger_accounts_no_delete BEFORE DELETE ON accounts
  FOR EACH ROW EXECUTE FUNCTION fn_reject_account_delete();

-- postgres-required-comments: no table comment. postgres-column-requires-trigger: no touch trigger.
CREATE TABLE orders (
  id uuid PRIMARY KEY,
  account_id uuid NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
  state invoice_state NOT NULL DEFAULT 'draft',
  states invoice_state[],
  total_cents integer NOT NULL,
  total_dollars numeric GENERATED ALWAYS AS (total_cents / 100.0) VIRTUAL,
  total_is_even boolean GENERATED ALWAYS AS (total_cents % 2 = 0) STORED,
  updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX orders_account_id_idx ON orders (account_id) WHERE state <> 'paid';
CREATE TRIGGER trigger_orders_no_delete BEFORE DELETE ON orders
  FOR EACH ROW EXECUTE FUNCTION fn_reject_order_delete();

CREATE TABLE order_lines (
  order_id uuid NOT NULL REFERENCES orders (id) ON DELETE CASCADE,
  line_no integer NOT NULL,
  PRIMARY KEY (order_id, line_no)
);
COMMENT ON TABLE order_lines IS 'One row per order line.';

-- postgres-finite-text-columns: a text column pinned to a closed set of values.
-- postgres-status-with-lifecycle-timestamps: a status that duplicates its own timestamps.
CREATE TABLE invoices (
  id uuid PRIMARY KEY,
  order_id uuid NOT NULL,
  line_no integer NOT NULL,
  status text NOT NULL CHECK (status IN ('draft', 'sent', 'paid')),
  sent_at timestamptz,
  failed_at timestamptz,
  CONSTRAINT invoices_line_fkey FOREIGN KEY (order_id, line_no)
    REFERENCES order_lines (order_id, line_no) ON DELETE CASCADE ON UPDATE RESTRICT
);
COMMENT ON TABLE invoices IS 'Billing documents.';

-- postgres-table-shape: banned *_history table name.
CREATE TABLE order_history (id uuid PRIMARY KEY);
COMMENT ON TABLE order_history IS 'Prior order states.';

-- postgres-required-predicates: a partitioned table; its key must bound every query.
CREATE TABLE events (
  id bigint GENERATED ALWAYS AS IDENTITY,
  created_at timestamptz NOT NULL,
  kind text NOT NULL,
  PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);
COMMENT ON TABLE events IS 'Append-only event log.';
CREATE TABLE events_2026 PARTITION OF events FOR VALUES FROM ('2026-01-01') TO ('2027-01-01');

CREATE TABLE nested_events (
  id bigint NOT NULL,
  created_at date NOT NULL,
  PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);
CREATE TABLE nested_events_2026 PARTITION OF nested_events
  FOR VALUES FROM ('2026-01-01') TO ('2027-01-01') PARTITION BY RANGE (created_at);
CREATE TABLE nested_events_2026_a PARTITION OF nested_events_2026
  FOR VALUES FROM ('2026-01-01') TO ('2026-07-01');

-- A partition leaf with its own foreign key is retained in the generated catalog.
CREATE TABLE partition_roots.local_key_parent (id uuid PRIMARY KEY) PARTITION BY HASH (id);
CREATE TABLE local_key_leaf PARTITION OF partition_roots.local_key_parent
  FOR VALUES WITH (MODULUS 2, REMAINDER 0);
ALTER TABLE local_key_leaf ADD CONSTRAINT local_key_leaf_account_fkey
  FOREIGN KEY (id) REFERENCES accounts(id);

CREATE VIEW open_orders AS SELECT id, account_id FROM orders WHERE state <> 'paid';
COMMENT ON VIEW open_orders IS 'Orders that are not yet paid.';
