// no-mistakes-disable-file postgres-pool-requires-error-listener -- The application owns these listeners.
import { Pool } from "pg";

export const pool = new Pool({ host: "db.example.invalid" });
