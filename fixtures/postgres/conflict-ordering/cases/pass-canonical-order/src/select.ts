import { query } from "@example/db";
// A pure SELECT needs no conflict projection, even when prepared locations are absent.
query("SELECT id FROM items");
