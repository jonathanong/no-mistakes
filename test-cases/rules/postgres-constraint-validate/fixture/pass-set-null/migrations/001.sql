ALTER TABLE child
  ADD CONSTRAINT child_topic_result_fk
  FOREIGN KEY (topic_id, result_id)
  REFERENCES parent (topic_id, result_id)
  ON DELETE SET NULL
  NOT VALID;
ALTER TABLE child
  VALIDATE CONSTRAINT child_topic_result_fk;
