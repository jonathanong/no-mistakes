-- A block comment ends E continuation even when it contains a newline.
COMMENT ON TABLE t IS E'a' /*
 block break */ 'ordinary';
CREATE INDEX after_block_break ON t(id);
-- Same-line strings also remain independent malformed values.
COMMENT ON TABLE t IS E'a' 'ordinary';
CREATE INDEX after_same_line ON t(id);
-- Unterminated continuation remains lexical failure rather than normalized success.
COMMENT ON TABLE t IS E'a'
 'b\'c
