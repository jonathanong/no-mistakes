-- Malformed trigger declarations exercise guarded recovery, never invented arguments.
;
SELECT 1;
CREATE;
CREATE TRIGGER missing_exec AFTER INSERT ON app.rows;
CREATE TRIGGER missing_kind AFTER INSERT ON app.rows EXECUTE;
CREATE TRIGGER wrong_kind AFTER INSERT ON app.rows EXECUTE SELECT;
CREATE TRIGGER missing_args AFTER INSERT ON app.rows EXECUTE FUNCTION app.audit;
CREATE TRIGGER negative_arg AFTER INSERT ON app.rows EXECUTE FUNCTION app.audit(-42);
CREATE TRIGGER expression_arg AFTER INSERT ON app.rows EXECUTE FUNCTION app.audit(1 + 2);
SELECT 2;
