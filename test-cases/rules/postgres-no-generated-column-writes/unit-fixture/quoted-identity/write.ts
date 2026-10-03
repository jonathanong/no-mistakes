import { write } from '@example/db'
// Both distinct quoted relations must survive one call's write deduplication.
write(`UPDATE "Orders" SET computed = 1; UPDATE "ORDERS" SET computed = 1`);
