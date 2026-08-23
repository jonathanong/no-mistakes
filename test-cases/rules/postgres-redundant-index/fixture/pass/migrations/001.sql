CREATE TABLE accounts (
  id uuid PRIMARY KEY,
  external_id uuid NOT NULL,
  region text NOT NULL
);

CREATE UNIQUE INDEX idx_accounts__external_id ON accounts (external_id);
CREATE INDEX idx_accounts__external_id__region ON accounts (external_id, region);
