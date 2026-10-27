FOR i IN 1..array_length(parts,1)-1 LOOP RAISE NOTICE '%',i;
FOR i IN 1..2 LOOP RAISE NOTICE '%',i;
FOR i IN low..high LOOP RAISE NOTICE '%',i;
FOR i IN 1 .. 2 LOOP RAISE NOTICE '%',i;
FOR i IN 1 ..2 LOOP RAISE NOTICE '%',i;
FOR i IN 1.. 2 LOOP RAISE NOTICE '%',i;
FOR i IN 1.5..2.5 LOOP RAISE NOTICE '%',i;
FOR i IN REVERSE -1..array_length(parts,1)-1 BY 2 LOOP RAISE NOTICE '%',i;
