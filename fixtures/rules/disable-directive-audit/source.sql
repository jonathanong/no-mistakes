-- no-mistakes-disable-file postgres-no-offset: leading file directive
-- no-mistakes-disable-file postgres-no-offset: shadowed duplicate should be unused
-- no-mistakes-disable-next-line postgres-no-offset: statement directive
SELECT 1;
SELECT 2; -- no-mistakes-disable-line postgres-no-offset: inline
SELECT '-- no-mistakes-disable-line fake-string';
/* -- no-mistakes-disable-line fake-block */
