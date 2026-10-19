CREATE TABLE items (id integer PRIMARY KEY, email text);
CREATE TABLE audited (id integer PRIMARY KEY);
CREATE TABLE users (id integer PRIMARY KEY, email text);
CREATE TRIGGER audit_replay BEFORE INSERT ON audited FOR EACH STATEMENT EXECUTE FUNCTION audit_event();
