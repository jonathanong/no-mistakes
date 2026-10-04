-- Invalid INSERT prefixes must not turn TABLE ONLY into a source query.
INSERT TABLE ONLY accounts;
INSERT INTO 1 TABLE ONLY accounts;
INSERT INTO public. TABLE ONLY accounts;
INSERT INTO archive AS 1 TABLE ONLY accounts;
INSERT INTO archive (id TABLE ONLY accounts;
INSERT INTO archive OVERRIDING SYSTEM TABLE ONLY accounts;
