import { write } from '@data-stores/psql'
// Both distinct quoted relations must survive one call's write deduplication.
write(`UPDATE "Orders" SET computed = 1; UPDATE "ORDERS" SET computed = 1`);
