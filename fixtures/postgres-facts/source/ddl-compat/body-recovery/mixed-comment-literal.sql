-- Implicit continuation requires another ordinary string, not a dollar string.
COMMENT ON TABLE accounts IS 'first'
$note$second$note$;
