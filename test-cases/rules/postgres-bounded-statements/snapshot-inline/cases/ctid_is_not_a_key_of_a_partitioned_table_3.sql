UPDATE events SET id = id WHERE ctid IN (SELECT ctid FROM events ORDER BY id LIMIT 5);
