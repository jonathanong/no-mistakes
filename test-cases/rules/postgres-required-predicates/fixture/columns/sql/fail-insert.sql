INSERT INTO archive (id) SELECT id FROM events WHERE kind = 'login';
