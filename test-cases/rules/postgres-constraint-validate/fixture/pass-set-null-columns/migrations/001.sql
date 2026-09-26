CREATE TABLE parent (
  topic_id uuid NOT NULL,
  result_id uuid NOT NULL,
  PRIMARY KEY (topic_id, result_id)
);
CREATE TABLE child (
  topic_id uuid NOT NULL,
  result_id uuid NOT NULL
);
ALTER TABLE child
  ADD CONSTRAINT child_topic_result_fk
  FOREIGN KEY (topic_id, result_id)
  REFERENCES parent (topic_id, result_id)
  ON DELETE SET NULL (result_id)
  NOT VALID;
ALTER TABLE child
  VALIDATE CONSTRAINT child_topic_result_fk;
