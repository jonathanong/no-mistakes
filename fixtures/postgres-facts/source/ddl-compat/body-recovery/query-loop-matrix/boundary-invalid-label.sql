-- A missing label terminator must not be skipped to expose an apparent loop.
<<broken FOR r IN SELECT 1 LOOP
