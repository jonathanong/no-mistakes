-- no-mistakes-disable-file postgres-redundant-index: fixture exception
CREATE TABLE events (
  id uuid PRIMARY KEY,
  topic_id uuid NOT NULL,
  created_at timestamptz NOT NULL
);

CREATE INDEX idx_events__topic_id ON events (topic_id);
CREATE INDEX idx_events__topic_id__created_at ON events (topic_id, created_at);
