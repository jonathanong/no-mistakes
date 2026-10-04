-- Only one correctly placed plpgsql LANGUAGE clause may follow a DO body.
DO $$ SELECT 1 $$ LANGUAGE plpgsql;
DO $$ SELECT 1 $$ LANGUAGE sql;
DO $$ SELECT 1 $$ LANGUAGE plpgsql trailing;
DO LANGUAGE plpgsql $$ SELECT 1 $$ LANGUAGE plpgsql;
