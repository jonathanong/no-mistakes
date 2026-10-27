SELECT jsonb_path_query(jsonb_agg(to_jsonb(o)), '$[*]') FROM orders o;
