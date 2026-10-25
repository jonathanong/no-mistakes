-- Newline-separated literals concatenate in PostgreSQL without an operator.
COMMENT ON TABLE accounts IS 'a. '
'b.';
COMMENT ON TABLE accounts IS $$text; SELECT FROM;$$;
COMMENT ON TABLE accounts IS $note$text; SELECT FROM;$note$;
COMMENT ON FUNCTION public.lookup_id(bigint) IS $$function text$$;
COMMENT ON FUNCTION public.lookup_id(bigint) IS $note$function text$note$;
COMMENT ON TRIGGER account_changed ON accounts IS $$trigger text$$;
