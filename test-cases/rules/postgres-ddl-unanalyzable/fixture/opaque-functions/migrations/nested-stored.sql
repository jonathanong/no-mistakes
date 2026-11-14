DO 'BEGIN
PERFORM 1;
CREATE FUNCTION this_function_name_is_longer_than_the_postgres_identifier_limit_of_sixty_three_bytes() RETURNS void LANGUAGE plpgsql AS $body$
BEGIN
ALTER TABLE hidden ADD COLUMN ignored text;
CREATE TABLE this_body_table_name_is_longer_than_the_postgres_identifier_limit_of_sixty_three_bytes (id int);
END
$body$;
ALTER TABLE visible ADD COLUMN reported text;
END';
