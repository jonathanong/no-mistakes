-- The two failed statements inside DO must retain separate heads and lines.
DO $$
BEGIN
  FROBULATE alpha;
  FROBULATE beta;
END
$$;
