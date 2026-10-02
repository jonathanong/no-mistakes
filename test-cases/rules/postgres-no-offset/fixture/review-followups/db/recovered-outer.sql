-- An earlier clause must not take a recovered statement's location.
SELECT id FROM older OFFSET 1;
DO $body$
SELECT id FROM posts OFFSET 2;
$body$;
DO $tag$SELECT id FROM posts OFFSET 4$tag$;
chr(83)||chr(69)||chr(76)||chr(69)||chr(67)||chr(84)||chr(32)||chr(49)||chr(32)||'OFFSET 3';
