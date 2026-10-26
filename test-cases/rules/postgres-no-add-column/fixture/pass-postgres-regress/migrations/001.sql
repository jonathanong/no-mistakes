-- add attribute: this comment must not be treated as SQL.
ALTER TABLE FKTABLE ADD FOREIGN KEY(ftest1, ftest2) REFERENCES pktable;
ALTER TABLE atacc2 ADD CONSTRAINT foo CHECK (test2 > 0);
ALTER DOMAIN dcomptype ADD CONSTRAINT c1 CHECK ((value).r <= (value).i);
ALTER TABLE itest3 ALTER COLUMN a ADD GENERATED ALWAYS AS IDENTITY;
ALTER GROUP regress_priv_group2 ADD USER regress_priv_user2;
ALTER TABLE gtest20a ADD CHECK (b < 50);
