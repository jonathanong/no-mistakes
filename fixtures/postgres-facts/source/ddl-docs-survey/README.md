# PostgreSQL 17 DDL command-family survey

Grammar source: https://www.postgresql.org/docs/17/sql-commands.html

Each row locates one audited example, not every grammar variant. The raw sqlparser result is diagnostic only; shared migration extraction also applies compatibility rewrites and routine recovery.

| Command | Example | Official reference |
| --- | --- | --- |
| ALTER AGGREGATE | families.sql:1 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alteraggregate.html) |
| ALTER COLLATION | families.sql:2 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-altercollation.html) |
| ALTER CONVERSION | families.sql:3 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterconversion.html) |
| ALTER DATABASE | families.sql:4 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterdatabase.html) |
| ALTER DEFAULT PRIVILEGES | Companion privileges fixture | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterdefaultprivileges.html) |
| ALTER DOMAIN | families.sql:5 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterdomain.html) |
| ALTER EVENT TRIGGER | survey.sql:6 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-altereventtrigger.html) |
| ALTER EXTENSION | families.sql:6 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterextension.html) |
| ALTER FOREIGN DATA WRAPPER | survey.sql:14 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterforeigndatawrapper.html) |
| ALTER FOREIGN TABLE | families.sql:7 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterforeigntable.html) |
| ALTER FUNCTION | families.sql:8 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterfunction.html) |
| ALTER GROUP | families.sql:9 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-altergroup.html) |
| ALTER INDEX | families.sql:10 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterindex.html) |
| ALTER LANGUAGE | families.sql:11 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterlanguage.html) |
| ALTER LARGE OBJECT | survey.sql:19 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterlargeobject.html) |
| ALTER MATERIALIZED VIEW | families.sql:12 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-altermaterializedview.html) |
| ALTER OPERATOR | families.sql:13 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alteroperator.html) |
| ALTER OPERATOR CLASS | families.sql:14 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alteropclass.html) |
| ALTER OPERATOR FAMILY | families.sql:15 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alteropfamily.html) |
| ALTER POLICY | families.sql:16 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterpolicy.html) |
| ALTER PROCEDURE | families.sql:17 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterprocedure.html) |
| ALTER PUBLICATION | families.sql:18 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterpublication.html) |
| ALTER ROLE | families.sql:19 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterrole.html) |
| ALTER ROUTINE | families.sql:20 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterroutine.html) |
| ALTER RULE | families.sql:21 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterrule.html) |
| ALTER SCHEMA | families.sql:22 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterschema.html) |
| ALTER SEQUENCE | families.sql:23 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-altersequence.html) |
| ALTER SERVER | families.sql:24 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterserver.html) |
| ALTER STATISTICS | families.sql:25 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterstatistics.html) |
| ALTER SUBSCRIPTION | families.sql:26 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-altersubscription.html) |
| ALTER SYSTEM | branches-valid.sql:27 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-altersystem.html) |
| ALTER TABLE | families.sql:27 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-altertable.html) |
| ALTER TABLESPACE | families.sql:28 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-altertablespace.html) |
| ALTER TEXT SEARCH CONFIGURATION | families.sql:29 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-altertsconfig.html) |
| ALTER TEXT SEARCH DICTIONARY | families.sql:30 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-altertsdictionary.html) |
| ALTER TEXT SEARCH PARSER | families.sql:31 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-altertsparser.html) |
| ALTER TEXT SEARCH TEMPLATE | families.sql:32 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-altertstemplate.html) |
| ALTER TRIGGER | families.sql:33 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-altertrigger.html) |
| ALTER TYPE | families.sql:34 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-altertype.html) |
| ALTER USER | families.sql:35 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alteruser.html) |
| ALTER USER MAPPING | families.sql:36 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterusermapping.html) |
| ALTER VIEW | families.sql:37 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-alterview.html) |
| CREATE ACCESS METHOD | survey.sql:1 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-create-access-method.html) |
| CREATE AGGREGATE | survey.sql:2 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createaggregate.html) |
| CREATE CAST | survey.sql:3 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createcast.html) |
| CREATE COLLATION | families.sql:38 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createcollation.html) |
| CREATE CONVERSION | survey.sql:4 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createconversion.html) |
| CREATE DATABASE | families.sql:39 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createdatabase.html) |
| CREATE DOMAIN | families.sql:40 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createdomain.html) |
| CREATE EVENT TRIGGER | survey.sql:5 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createeventtrigger.html) |
| CREATE EXTENSION | families.sql:41 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createextension.html) |
| CREATE FOREIGN DATA WRAPPER | survey.sql:13 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createforeigndatawrapper.html) |
| CREATE FOREIGN TABLE | families.sql:42 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createforeigntable.html) |
| CREATE FUNCTION | families.sql:43 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createfunction.html) |
| CREATE GROUP | families.sql:44 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-creategroup.html) |
| CREATE INDEX | families.sql:45 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createindex.html) |
| CREATE LANGUAGE | families.sql:46 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createlanguage.html) |
| CREATE MATERIALIZED VIEW | families.sql:47 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-creatematerializedview.html) |
| CREATE OPERATOR | families.sql:48 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createoperator.html) |
| CREATE OPERATOR CLASS | families.sql:49 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createopclass.html) |
| CREATE OPERATOR FAMILY | families.sql:50 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createopfamily.html) |
| CREATE POLICY | families.sql:51 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createpolicy.html) |
| CREATE PROCEDURE | families.sql:52 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createprocedure.html) |
| CREATE PUBLICATION | families.sql:53 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createpublication.html) |
| CREATE ROLE | families.sql:54 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createrole.html) |
| CREATE RULE | families.sql:55 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createrule.html) |
| CREATE SCHEMA | families.sql:56 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createschema.html) |
| CREATE SEQUENCE | families.sql:57 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createsequence.html) |
| CREATE SERVER | survey.sql:15 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createserver.html) |
| CREATE STATISTICS | families.sql:58 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createstatistics.html) |
| CREATE SUBSCRIPTION | families.sql:59 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createsubscription.html) |
| CREATE TABLE | families.sql:60 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createtable.html) |
| CREATE TABLE AS | families.sql:61 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createtableas.html) |
| CREATE TABLESPACE | families.sql:62 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createtablespace.html) |
| CREATE TEXT SEARCH CONFIGURATION | families.sql:63 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createtsconfig.html) |
| CREATE TEXT SEARCH DICTIONARY | survey.sql:10 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createtsdictionary.html) |
| CREATE TEXT SEARCH PARSER | survey.sql:11 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createtsparser.html) |
| CREATE TEXT SEARCH TEMPLATE | survey.sql:12 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createtstemplate.html) |
| CREATE TRANSFORM | survey.sql:8 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createtransform.html) |
| CREATE TRIGGER | families.sql:107 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createtrigger.html) |
| CREATE TYPE | families.sql:64 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createtype.html) |
| CREATE USER | survey.sql:16 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createuser.html) |
| CREATE USER MAPPING | survey.sql:16 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createusermapping.html) |
| CREATE VIEW | families.sql:66 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-createview.html) |
| DROP ACCESS METHOD | families.sql:67 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-drop-access-method.html) |
| DROP AGGREGATE | families.sql:68 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropaggregate.html) |
| DROP CAST | families.sql:69 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropcast.html) |
| DROP COLLATION | families.sql:70 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropcollation.html) |
| DROP CONVERSION | families.sql:71 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropconversion.html) |
| DROP DATABASE | families.sql:72 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropdatabase.html) |
| DROP DOMAIN | families.sql:73 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropdomain.html) |
| DROP EVENT TRIGGER | survey.sql:7 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropeventtrigger.html) |
| DROP EXTENSION | families.sql:74 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropextension.html) |
| DROP FOREIGN DATA WRAPPER | families.sql:75 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropforeigndatawrapper.html) |
| DROP FOREIGN TABLE | families.sql:76 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropforeigntable.html) |
| DROP FUNCTION | families.sql:77 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropfunction.html) |
| DROP GROUP | families.sql:78 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropgroup.html) |
| DROP INDEX | families.sql:79 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropindex.html) |
| DROP LANGUAGE | families.sql:80 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-droplanguage.html) |
| DROP MATERIALIZED VIEW | families.sql:81 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropmaterializedview.html) |
| DROP OPERATOR | families.sql:82 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropoperator.html) |
| DROP OPERATOR CLASS | families.sql:83 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropopclass.html) |
| DROP OPERATOR FAMILY | families.sql:84 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropopfamily.html) |
| DROP OWNED | survey.sql:21 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-drop-owned.html) |
| DROP POLICY | families.sql:85 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-droppolicy.html) |
| DROP PROCEDURE | families.sql:86 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropprocedure.html) |
| DROP PUBLICATION | families.sql:87 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-droppublication.html) |
| DROP ROLE | families.sql:88 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-droprole.html) |
| DROP ROUTINE | families.sql:89 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-droproutine.html) |
| DROP RULE | families.sql:90 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-droprule.html) |
| DROP SCHEMA | families.sql:91 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropschema.html) |
| DROP SEQUENCE | families.sql:92 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropsequence.html) |
| DROP SERVER | families.sql:93 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropserver.html) |
| DROP STATISTICS | families.sql:94 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropstatistics.html) |
| DROP SUBSCRIPTION | families.sql:95 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropsubscription.html) |
| DROP TABLE | families.sql:96 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-droptable.html) |
| DROP TABLESPACE | families.sql:97 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-droptablespace.html) |
| DROP TEXT SEARCH CONFIGURATION | families.sql:98 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-droptsconfig.html) |
| DROP TEXT SEARCH DICTIONARY | families.sql:99 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-droptsdictionary.html) |
| DROP TEXT SEARCH PARSER | families.sql:100 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-droptsparser.html) |
| DROP TEXT SEARCH TEMPLATE | families.sql:101 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-droptstemplate.html) |
| DROP TRANSFORM | survey.sql:9 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-droptransform.html) |
| DROP TRIGGER | families.sql:102 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-droptrigger.html) |
| DROP TYPE | families.sql:103 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-droptype.html) |
| DROP USER | families.sql:104 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropuser.html) |
| DROP USER MAPPING | families.sql:105 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropusermapping.html) |
| DROP VIEW | families.sql:106 | [PostgreSQL 17](https://www.postgresql.org/docs/17/sql-dropview.html) |
