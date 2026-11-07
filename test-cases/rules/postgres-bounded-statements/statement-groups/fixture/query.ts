import { query } from "@example/db";
query("SELECT id FROM items"); query("SELECT id FROM items");
query("SELECT id FROM items;\nSELECT id FROM items");
