CREATE TABLE "Orders" ("ID" uuid PRIMARY KEY, "CreatedAt" timestamptz GENERATED ALWAYS AS (uuid_extract_timestamp("ID")) STORED);
