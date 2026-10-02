-- Foreign-language body text must not consume a later PostgreSQL declaration.
DO LANGUAGE plpython3u $$
CREATE TABLE actual_items (value int);
$$;
DO LANGUAGE "plpgsql" $$
CREATE TABLE quoted_supported_items (value int);
$$;
CREATE TABLE actual_items (value int);
DO LANGUAGE plpgsql $$
CREATE TABLE supported_items (value int);
$$;
DO LANGUAGE "PLPGSQL" $$
CREATE TABLE skipped_items (value int);
$$;
DO LANGUAGE PLPGSQL $$
CREATE TABLE uppercase_supported_items (value int);
$$;
