import type { ExecutorOptions } from "./db";
import type { SqlStatement } from "./sql";

export declare function lookup(
  statement: string | SqlStatement,
  options?: ExecutorOptions,
): Promise<unknown>;
