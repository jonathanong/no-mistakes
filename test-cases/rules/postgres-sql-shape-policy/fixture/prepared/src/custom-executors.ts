import { last, first } from "@custom/database";
first(`SELECT id FROM orders WHERE id NOT IN (SELECT id FROM archived)`);
last(`SELECT id FROM orders WHERE id NOT IN (SELECT id FROM archived)`);
