CREATE TYPE item_identity AS (post_id uuid, content_sha256 text, policy_revision integer);
CREATE TABLE items (post_id uuid, content_sha256 text, policy_revision integer, id uuid);
CREATE INDEX items_identity_idx ON items (((post_id, content_sha256, policy_revision)::item_identity));
