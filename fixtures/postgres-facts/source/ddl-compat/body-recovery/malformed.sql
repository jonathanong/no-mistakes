SELECT abs(id) INTO STRICT first_id FROM accounts;
SELECT abs(id) INTO STRICT first_id, second_id FROM accounts;
COMMENT ON TABLE accounts IS 'a' 'b';
COMMENT ON TABLE accounts IS $$text$$ trailing;
COMMENT ON FUNCTION public.lookup_id(bigint) IS $$text$$ trailing;
COMMENT ON TABLE IS $$text$$;
BEGIN SELECT abs(id) INTO STRICT first_id, second_id FROM accounts; COMMIT;
'SELECT abs(id) INTO STRICT first_id, second_id FROM accounts';
