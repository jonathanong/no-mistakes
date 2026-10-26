-- ALTER_TABLE and CREATE_TABLE are comment banners.
-- CREATE VIEW does not support USING here.
CREATE USER regress_alter_table_user1;
CREATE ACCESS METHOD bogus TYPE INDEX HANDLER heap_tableam_handler;
CREATE TABLE real_table (id int);
