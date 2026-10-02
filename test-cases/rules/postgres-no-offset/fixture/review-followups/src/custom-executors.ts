import { last, first } from "@custom/database";
first(`SELECT id FROM orders OFFSET 1`);
last(`SELECT id FROM orders OFFSET 0`);
