import { query } from '@example/db';
query('SELECT id FROM selected_items FOR UPDATE');
