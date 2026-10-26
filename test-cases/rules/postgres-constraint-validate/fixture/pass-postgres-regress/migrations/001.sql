-- Unquoted PostgreSQL identifiers fold to lower case for validation pairing.
ALTER TABLE attmp3 ADD CONSTRAINT IDENTITY CHECK (b = boo(b)) NOT VALID;
ALTER TABLE attmp3 VALIDATE CONSTRAINT identity;
-- An unnamed CHECK receives the table_column_check default name.
ALTER TABLE nv_child_2011 ADD CHECK (d BETWEEN '2011-01-01'::date AND '2011-12-31'::date) NOT VALID;
ALTER TABLE nv_child_2011 VALIDATE CONSTRAINT nv_child_2011_d_check;
