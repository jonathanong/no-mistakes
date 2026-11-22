import type {
  LockInventoryReport,
  PostgresLockInventoryOptions,
} from "./postgres-lock-inventory-types";
import type { WithInvocationOptions, WithInvocationResult } from "./invocation-types";

export function postgresLockInventory(
  options?: WithInvocationOptions<PostgresLockInventoryOptions>,
): Promise<WithInvocationResult<LockInventoryReport>>;
