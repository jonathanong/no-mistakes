-- Encoded migrations retain numeric identity after source reconstruction.
'SELECT * FROM orders LIMIT ' || chr(48) || 'xFF';
'SELECT * FROM orders LIMIT ' || '0xF_F';
'SELECT * FROM orders LIMIT X' || chr(39) || 'FF' || chr(39);
'SELECT * FROM orders LIMIT ' || $$0x10$$;
SELECT * FROM orders LIMIT X'FF';
SELECT * FROM orders LIMIT 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF;
