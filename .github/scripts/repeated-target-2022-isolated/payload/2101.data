-- These parser-supported function forms have no complete dependency proof.
-- Keep each fail-closed path covered even though the SQL is not an index DDL.
INSERT INTO users (email) VALUES ('a')
ON CONFLICT ((ARRAY(SELECT 1))) DO UPDATE SET email = 'b';
INSERT INTO users (email) VALUES ('a')
ON CONFLICT ((array_agg(DISTINCT email))) DO UPDATE SET email = 'b';
INSERT INTO users (email) VALUES ('a')
ON CONFLICT ((lower(value => email))) DO UPDATE SET email = 'b';
