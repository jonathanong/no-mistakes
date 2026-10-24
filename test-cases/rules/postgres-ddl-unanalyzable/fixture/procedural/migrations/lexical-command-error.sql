-- A lexer failure is not proof of a successfully handled control-only body.
DO $body$
DECLARE
  value_text text;
BEGIN
  value_text := 'ok';
  value_text := 'unterminated;
END
$body$;
FROBULATE after_lexical_body;

-- Recovered SQL before the bad tail must not hide the lexical failure.
DO $broken$
BEGIN
  CREATE TABLE before_lexical_failure (id integer);
  value_text := 'unterminated;
END
$broken$;
FROBULATE after_partial_lexical_body;
