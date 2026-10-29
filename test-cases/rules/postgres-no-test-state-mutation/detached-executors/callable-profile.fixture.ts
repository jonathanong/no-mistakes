import { customExecutor, start } from '@example/db';
const { query } = customExecutor;
query('DROP TABLE configured_callable_property');
const bound = start.query.bind(start);
bound('DROP TABLE configured_factory_property');
