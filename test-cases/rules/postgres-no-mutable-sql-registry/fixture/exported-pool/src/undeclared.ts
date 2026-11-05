import { unknownPool } from "@example/db";

unknownPool.query("/* users/load */ SELECT id FROM app.users WHERE id = $1", ["id"]);
