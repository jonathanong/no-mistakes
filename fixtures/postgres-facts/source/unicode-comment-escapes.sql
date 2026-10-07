-- Custom escapes must decode from original spelling, not default-decoded tokens.
COMMENT ON FUNCTION custom_doc() IS U&'d!0061t!+000061!! it''''s \path' UESCAPE '!';
COMMENT ON TABLE "Schéma"."T" IS U&'snow !96EA' UESCAPE '!';
COMMENT ON VIEW default_doc IS U&'d\0061t\+000061\\';
COMMENT ON TABLE surrogate_doc IS U&'!D83D!DE00 !+00D83D!+00DE00 !D83D!+00DE00 !+00D83D!DE00' UESCAPE '!';
COMMENT ON TABLE nonascii_escape IS U&'§0061§§' UESCAPE '§';
COMMENT ON TABLE clear_doc IS NULL;
COMMENT ON TABLE dollar_doc IS $doc$!0041\0041$doc$;
DO $body$ BEGIN
  IF true THEN
    COMMENT ON FUNCTION nested_doc() IS U&'!0041' UESCAPE '!';
    COMMENT ON TABLE nested_table IS U&'!0042' UESCAPE '!';
  END IF;
END $body$;
-- Every invalid literal keeps the independent neighbor following it.
COMMENT ON TABLE bad_doc IS U&'text' UESCAPE '0';
SELECT 1 AS neighbor;
COMMENT ON TABLE bad_doc IS U&'text' UESCAPE '+';
SELECT 2 AS neighbor;
COMMENT ON TABLE bad_doc IS U&'text' UESCAPE '''';
SELECT 3 AS neighbor;
COMMENT ON TABLE bad_doc IS U&'text' UESCAPE '"';
SELECT 4 AS neighbor;
COMMENT ON TABLE bad_doc IS U&'text' UESCAPE ' ';
SELECT 5 AS neighbor;
COMMENT ON TABLE bad_doc IS U&'text' UESCAPE '';
SELECT 6 AS neighbor;
COMMENT ON TABLE bad_doc IS U&'text' UESCAPE '!!';
SELECT 7 AS neighbor;
COMMENT ON TABLE bad_doc IS U&'text' UESCAPE identifier;
SELECT 8 AS neighbor;
COMMENT ON TABLE bad_doc IS U&'!004Z' UESCAPE '!';
SELECT 9 AS neighbor;
COMMENT ON TABLE bad_doc IS U&'!0000' UESCAPE '!';
SELECT 10 AS neighbor;
COMMENT ON TABLE bad_doc IS U&'!D800' UESCAPE '!';
SELECT 11 AS neighbor;
COMMENT ON TABLE bad_doc IS U&'\00ZZ';
SELECT 12 AS neighbor;
COMMENT ON TABLE bad_doc IS 'plain' UESCAPE '!';
SELECT 13 AS neighbor;
