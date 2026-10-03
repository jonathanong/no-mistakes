import { write } from '@example/db'
write(`UPDATE orders SET created_at = now(), updated_at = now()`);
