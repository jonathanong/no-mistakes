DROP INDEX CONCURRENTLY IF EXISTS idx_history__topic_id;
CREATE INDEX idx_history__topic_id_id ON history (topic_id, id);
