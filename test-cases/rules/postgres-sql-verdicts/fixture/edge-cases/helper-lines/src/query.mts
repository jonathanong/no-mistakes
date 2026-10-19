import { query } from "@example/db";
import { statement } from "./fragment.mts";

query(statement()); // The imported SQL is known, but this table has no catalog owner.
