import { pool } from "./missing.js";

pool.on("error", (error) => console.error(error));
