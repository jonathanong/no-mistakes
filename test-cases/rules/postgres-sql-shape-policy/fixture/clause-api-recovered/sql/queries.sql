-- Recovery wrappers do not create SELECT-list clauses, but nested real queries do.
DO $$ BEGIN IF probe_boundary() THEN PERFORM probe_boundary(); END IF; RETURN probe_boundary(); END $$;
DO $$ BEGIN PERFORM (SELECT probe_boundary()); END $$;
