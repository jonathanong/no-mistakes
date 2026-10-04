-- TABLE ONLY is the INSERT query source, after target aliases and identity overrides.
INSERT INTO archive TABLE ONLY accounts;
INSERT INTO archive AS destination TABLE ONLY "Accounts";
INSERT INTO archive (id) OVERRIDING SYSTEM VALUE TABLE ONLY public.accounts;
INSERT INTO archive AS destination (id) OVERRIDING USER VALUE TABLE ONLY public."Accounts";
INSERT INTO public.archive TABLE ONLY accounts;
