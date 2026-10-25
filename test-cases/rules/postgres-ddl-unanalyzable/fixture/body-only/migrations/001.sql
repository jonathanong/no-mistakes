-- An opaque body command must advise even when every rule has failure severity.
DO $body$
BEGIN
  FROBULATE unsupported_body;
END
$body$;
