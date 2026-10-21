import { query } from "@app/db";
import { statement } from "./shared.mjs";
import { cyclic } from "./cycle-a.mjs";
query("SELECT 5");
query(statement());
query(cyclic());
