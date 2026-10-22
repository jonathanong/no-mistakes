import { query } from "@example/db";
import type { query as TypeOnlyQuery, Executor } from "@example/db";
import defaultDb from "@example/db";
import * as db from "@example/db";
import * as other from "@unrelated/db";
import { createExecutor } from "@example/db/subpath";
import type { Executor as SubExecutor } from "@example/db/subpath";
export const unrelated = 1;
