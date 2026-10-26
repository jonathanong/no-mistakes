import { PoolClient } from "@example/db";

const connection = new PoolClient();
connection.query("SELECT 1"); // diagnostic
