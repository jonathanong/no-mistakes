COMMENT ON TABLE t IS E'a'
 'b\'c';
COMMENT ON TABLE t IS E'雪' -- line comments keep the inherited E state
 '\'quoted\'';
COMMENT ON TABLE t IS E'first\'segment'
 '\\path';
COMMENT ON TABLE t IS E'a'
 'b''c\\';
COMMENT ON TABLE t IS E'x'
 'line\n'
 'last\'quote';
-- Lexer-looking content in ordinary/dollar strings must remain literal text.
COMMENT ON TABLE t IS 'E''a''
 ''plain''';
COMMENT ON TABLE t IS $tag$E'a'
 'b\'c'$tag$;
/* E'a' nested /* 'fake\'quote' */ must not start a continuation. */
COMMENT ON TABLE foo$tag$ IS E'after comment'
 ' \"double\" and \'single\'';
CREATE TABLE continuation_neighbor (label TEXT DEFAULT E'base'
 ' plus\'quote');
CREATE INDEX after_escaped_quotes ON continuation_neighbor(label);
COMMENT ON TABLE t IS $_tag_$tag with underscores$_tag_$;
-- A comment at EOF must not look for an absent newline.