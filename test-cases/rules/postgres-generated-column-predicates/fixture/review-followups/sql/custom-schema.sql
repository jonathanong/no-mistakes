CREATE TABLE custom_events (slug text, derived text GENERATED ALWAYS AS (other_fn(slug)) STORED);
