import * as psql from "@example/db";

// The alias intentionally has no declared executor type.
export const writePool = psql.writePool;
