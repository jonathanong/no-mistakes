import { write } from '@data-stores/psql'
write(`UPDATE orders SET created_at = now(), updated_at = now()`);
