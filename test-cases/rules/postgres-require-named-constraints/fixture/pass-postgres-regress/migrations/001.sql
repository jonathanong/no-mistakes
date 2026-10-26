-- FOREIGN KEY CONSTRAINT adding TEST is commentary.
-- Once check_b is added to the parent, validation follows.
ALTER TABLE onek ADD CONSTRAINT onek_check_constraint CHECK (unique1 >= 0);
ALTER TABLE p1 ADD CONSTRAINT p1chk CHECK (ff1 > 0) NO INHERIT;
