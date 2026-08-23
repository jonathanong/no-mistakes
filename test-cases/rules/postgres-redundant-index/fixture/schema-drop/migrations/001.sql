CREATE INDEX idx_events__topic_id ON public.events (topic_id);
CREATE INDEX idx_events__topic_id__created_at ON public.events (topic_id, created_at);
