import { query, read, write } from '@example/db';
const { query: fromQuery } = query;
fromQuery('DROP TABLE callable_query_property');
const { query: fromRead } = read;
fromRead('DROP TABLE callable_read_property');
const bound = write.query.bind(write);
bound('DROP TABLE callable_write_property');
