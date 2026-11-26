CREATE TABLE inline_case (
  id uuid PRIMARY KEY,
  topic_id uuid NOT NULL,
  created_at timestamptz NOT NULL
);

CREATE INDEX idx_inline_case__topic_id ON inline_case (topic_id); -- no-mistakes-disable-line postgres-redundant-index: fixture exception
CREATE INDEX idx_inline_case__topic_id__created_at ON inline_case (topic_id, created_at);

CREATE TABLE following_case (
  id uuid PRIMARY KEY,
  topic_id uuid NOT NULL,
  created_at timestamptz NOT NULL
);

-- no-mistakes-disable-next-line postgres-redundant-index: fixture exception
CREATE INDEX idx_following_case__topic_id ON following_case (topic_id);
CREATE INDEX idx_following_case__topic_id__created_at ON following_case (topic_id, created_at);

CREATE TABLE control_case (
  id uuid PRIMARY KEY,
  topic_id uuid NOT NULL,
  created_at timestamptz NOT NULL
);

CREATE INDEX idx_control_case__topic_id ON control_case (topic_id);
CREATE INDEX idx_control_case__topic_id__created_at ON control_case (topic_id, created_at);
