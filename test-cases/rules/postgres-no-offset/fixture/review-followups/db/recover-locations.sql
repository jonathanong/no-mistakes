-- Ignored unsupported text must not consume the real OFFSET source location.
UNSUPPORTED OFFSET 99;
SELECT id FROM orders OFFSET 0;
BEGIN CREATE VIEW later AS SELECT id FROM orders OFFSET 1;
