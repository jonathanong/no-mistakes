-- A nested SQL call can source arbitrarily many JSON values from server data.
SELECT a.email FROM jsonb_path_query(app.load_payload($1), '$[*]') x(value) JOIN accounts a ON a.email = x.value::text;
-- PostgreSQL expression-named arguments must receive the same data-dependency check.
SELECT a.email FROM jsonb_path_query(target => (SELECT jsonb_agg(payload) FROM batches), path => '$[*]') x(value) JOIN accounts a ON a.email = x.value::text;
-- This parser-only wildcard call is invalid for the builtin, but cannot prove caller sizing.
SELECT a.email FROM jsonb_path_query(*) x(value) JOIN accounts a ON a.email = x.value::text;
-- Parser token types come from server registration, not the caller-supplied parser name.
SELECT a.email FROM ts_token_type($1) x JOIN accounts a ON a.email = x.alias;
-- A caller-supplied JSON value still bounds the keyed join.
SELECT a.email FROM jsonb_path_query(target => $1::jsonb, path => '$[*]') x(value) JOIN accounts a ON a.email = x.value::text;
