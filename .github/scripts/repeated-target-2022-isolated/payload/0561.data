-- Keep the two LOCK statements adjacent: an inverted inclusive range can be empty.
DO $$ BEGIN IF true THEN LOCK TABLE sample_parent IN SHARE ROW EXCLUSIVE MODE;LOCK TABLE sample_child IN SHARE ROW EXCLUSIVE MODE; END IF; END $$;
