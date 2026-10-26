-- Rules with action lists are kept whole, even while their grammar is unsupported.
CREATE RULE r AS ON INSERT TO t DO INSTEAD (NOTIFY foo; NOTIFY bar);
CREATE OR REPLACE RULE r AS ON INSERT TO t DO ALSO (NOTIFY foo; NOTIFY bar);
CREATE TABLE after_rule (id int);
