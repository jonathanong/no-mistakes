-- Dynamic query expressions use independent result targets and parameter expressions.
EXECUTE format('SELECT %I FROM accounts', column_name) INTO selected_id;
EXECUTE format('SELECT %I FROM accounts', column_name) INTO STRICT selected_id;
EXECUTE format('SELECT %I, id FROM accounts', column_name) INTO row_value.id, selected_id USING abs(parameter_value), parameter_value + 1;
EXECUTE format('SELECT %I FROM accounts WHERE id = $1', column_name) INTO STRICT selected_id USING parameter_value;
EXECUTE format('SELECT %I FROM accounts WHERE id = $1', column_name) USING parameter_value;
EXECUTE format('SELECT %I FROM accounts WHERE id = $1', column_name) USING abs(parameter_value), ARRAY[1, 2];
