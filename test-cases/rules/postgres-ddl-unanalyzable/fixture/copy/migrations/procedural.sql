-- A COPY recovered from a DO body is not a top-level COPY data block.
DO $$ COPY accounts (id) FROM STDIN; $$;
\.
