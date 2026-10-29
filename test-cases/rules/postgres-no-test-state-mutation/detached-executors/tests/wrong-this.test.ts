import { pool } from '@example/db';
const wrongThis = pool.query.bind(otherReceiver);
wrongThis('DROP TABLE uncertain_receiver');
