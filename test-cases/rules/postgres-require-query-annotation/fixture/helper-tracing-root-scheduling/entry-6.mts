import { query } from "@app/db";
import { statement } from "./shared.mjs";
import { cyclic } from "./cycle-a.mjs";
query("SELECT 6");
query(statement());
query(cyclic());
