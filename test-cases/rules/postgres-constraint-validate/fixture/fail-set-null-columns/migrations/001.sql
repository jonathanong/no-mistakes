-- Names differ on purpose. Column-specific SET NULL must still be seen as a
-- named NOT VALID add, and the other VALIDATE CONSTRAINT stays unmatched.
ALTER TABLE child
  ADD CONSTRAINT child_topic_result_fk
  FOREIGN KEY (topic_id, result_id)
  REFERENCES parent (topic_id, result_id)
  ON DELETE SET NULL (result_id)
  NOT VALID;
ALTER TABLE child
  VALIDATE CONSTRAINT child_other_fk;
