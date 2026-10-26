-- Recognition must validate the query expression, assignable targets, and parameters.
EXECUTE format('SELECT %I', column_name) + INTO selected_id;
EXECUTE format('SELECT %I', column_name) INTO;
EXECUTE format('SELECT %I', column_name) INTO STRICT;
EXECUTE format('SELECT %I', column_name) INTO selected_id + 1;
EXECUTE format('SELECT %I', column_name) INTO selected_id,;
EXECUTE format('SELECT %I', column_name) INTO selected_id USING;
EXECUTE format('SELECT %I', column_name) INTO STRICT selected_id USING parameter_value +;
EXECUTE format('SELECT %I', column_name) USING parameter_value,;
EXECUTE format('SELECT %I', column_name) USING (parameter_value;
