SELECT x.id FROM docs d, LATERAL XMLTABLE('/row' PASSING d.xml COLUMNS id integer PATH '@id') AS x;
SELECT x.id FROM docs d, LATERAL XMLTABLE('/row' PASSING d.xml COLUMNS id integer, name text PATH 'name') AS x;
