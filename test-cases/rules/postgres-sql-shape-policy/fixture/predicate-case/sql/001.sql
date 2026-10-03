SELECT id FROM jobs WHERE status = 'IDLE' AND id > $1 ORDER BY id LIMIT $2;
SELECT id FROM jobs WHERE STATUS = 'idle' AND id > $1 ORDER BY id LIMIT $2;
