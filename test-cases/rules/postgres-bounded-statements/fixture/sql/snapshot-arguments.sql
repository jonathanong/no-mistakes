-- Server-derived snapshots can contain every active transaction, regardless of caller inputs.
SELECT a.email FROM pg_snapshot_xip(pg_current_snapshot()) x(xid) JOIN accounts a ON a.email = x.xid::text;
SELECT a.email FROM txid_snapshot_xip(txid_current_snapshot()) x(xid) JOIN accounts a ON a.email = x.xid::text;
SELECT a.email FROM pg_catalog.pg_snapshot_xip(pg_catalog.pg_current_snapshot()) x(xid) JOIN accounts a ON a.email = x.xid::text;
SELECT a.email FROM pg_snapshot_xip(COALESCE($1::pg_snapshot, pg_current_snapshot())) x(xid) JOIN accounts a ON a.email = x.xid::text;
SELECT a.email FROM pg_snapshot_xip(app.load_snapshot($1)) x(xid) JOIN accounts a ON a.email = x.xid::text;
-- A caller-supplied snapshot, including NULL, remains caller-sized.
SELECT a.email FROM pg_snapshot_xip($1::pg_snapshot) x(xid) JOIN accounts a ON a.email = x.xid::text;
SELECT a.email FROM pg_snapshot_xip(NULL) x(xid) JOIN accounts a ON a.email = x.xid::text;
