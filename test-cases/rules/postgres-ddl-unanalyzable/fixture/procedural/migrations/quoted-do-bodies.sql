-- Both supported SQL string forms contain a complete procedural body.
DO 'BEGIN PERFORM 1; END';
DO E'BEGIN PERFORM 2; END' LANGUAGE plpgsql;
