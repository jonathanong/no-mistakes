COMMENT ON TABLE t IS E'foo'
 'bar';
COMMENT ON TABLE t IS E'first'
 '\nsecond\tend\r\b\f\\\z';
COMMENT ON TABLE t IS E''
 '\101\x42\u0043\U00000044\xZ';
COMMENT ON TABLE t IS E''
 '\303\251雪';
COMMENT ON TABLE t IS E''
 '\xFF';
COMMENT ON TABLE t IS E''
 '\u12';
COMMENT ON TABLE t IS E''
 '\uD800';
COMMENT ON TABLE t IS E''
 '\000';
COMMENT ON TABLE t IS E''
 '\777';
CREATE TABLE escaped_default (value text DEFAULT E'one'
 'two');
INSERT INTO escaped_default VALUES (E'left'
 'right');
