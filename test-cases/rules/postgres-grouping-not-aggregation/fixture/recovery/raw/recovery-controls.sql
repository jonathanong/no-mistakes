-- complete statement before the unsupported chunk
SELECT DISTINCT id FROM posts;
-- the unsupported chunk produces a recovery marker
INSERT INTO current_posts SELECT DISTINCT id FROM posts WHERE id::text COLLATE "C" = $1;
-- complete statement after the unsupported chunk
SELECT DISTINCT id FROM posts;
