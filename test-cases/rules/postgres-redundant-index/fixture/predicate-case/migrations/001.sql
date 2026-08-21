CREATE INDEX idx_events__topic_id ON events (topic_id) WHERE status = 'ACTIVE';
CREATE INDEX idx_events__topic_id__created_at ON events (topic_id, created_at) WHERE status = 'active';
