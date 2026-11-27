const db = require("@example/db");
const { read: r } = db;
import e = require("@example/db");

declare const items: string[];
await Promise.all(items.map((id) => db.read("SELECT 1", [id])));
await Promise.all(items.map((id) => r("SELECT 1", [id])));
await Promise.all(items.map((id) => e.read("SELECT 1", [id])));
