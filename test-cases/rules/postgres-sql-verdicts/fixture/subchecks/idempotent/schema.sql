CREATE TABLE items (id integer PRIMARY KEY, note text, name text, name_key text GENERATED ALWAYS AS (lower(name)) STORED UNIQUE);
CREATE TABLE audited (id integer PRIMARY KEY, note text);
CREATE TRIGGER audit_replay BEFORE INSERT ON audited FOR EACH ROW EXECUTE FUNCTION audit_event();
