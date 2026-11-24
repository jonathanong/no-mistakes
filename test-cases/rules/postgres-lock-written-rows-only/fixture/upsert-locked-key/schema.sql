CREATE TABLE jobs (
  id integer PRIMARY KEY,
  state text UNIQUE,
  touched_at text
);

CREATE TABLE job_audit (
  id integer PRIMARY KEY,
  state text
);
