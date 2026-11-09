CREATE TABLE direct_tags (
  item_id uuid NOT NULL,
  tag_id uuid NOT NULL,
  PRIMARY KEY (item_id, tag_id)
);
CREATE TABLE alias_tags (
  item_id uuid NOT NULL,
  tag_id uuid NOT NULL,
  PRIMARY KEY (item_id, tag_id)
);
CREATE TABLE topic_memberships (
  post_id uuid NOT NULL,
  topic_id uuid NOT NULL,
  PRIMARY KEY (post_id, topic_id)
);
CREATE TABLE live_topics (
  topic_id uuid NOT NULL,
  deleted_at timestamptz
);
CREATE UNIQUE INDEX live_topics_topic_id_live_key
  ON live_topics (topic_id) WHERE deleted_at IS NULL;
CREATE TABLE nullable_slugs (
  slug text UNIQUE
);
CREATE TABLE request_items (
  input_order integer PRIMARY KEY,
  slug text NOT NULL
);
CREATE TABLE posts (
  id uuid PRIMARY KEY,
  slug text NOT NULL UNIQUE,
  author_id uuid NOT NULL,
  created_at timestamptz NOT NULL,
  parent_id uuid
);
CREATE TABLE current_posts (id uuid PRIMARY KEY);
CREATE TABLE archived_posts (id uuid PRIMARY KEY);
CREATE TABLE post_reads (
  id uuid PRIMARY KEY,
  post_id uuid NOT NULL,
  reader_id uuid NOT NULL,
  created_at timestamptz NOT NULL
);
CREATE TABLE blocked_posts (
  id uuid PRIMARY KEY,
  post_id uuid NOT NULL,
  active boolean NOT NULL
);
CREATE TABLE post_tags (
  id uuid PRIMARY KEY,
  post_id uuid NOT NULL,
  tag_id uuid NOT NULL,
  created_at timestamptz NOT NULL
);
CREATE TABLE ratings (
  topic_id uuid NOT NULL,
  score integer NOT NULL
);

CREATE TABLE nullable_equal_slugs (slug text UNIQUE NULLS NOT DISTINCT);
