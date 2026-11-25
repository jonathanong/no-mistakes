// no-mistakes-disable-next-line no-cross-package-relative-imports: This fixture exercises the package migration.
import { receiptStatus } from "../../shared/src/status.mjs";
import type { Receipt } from "../../shared/src/types.mjs"; // no-mistakes-disable-line no-cross-package-relative-imports: The types are moving to the public entry.
export const receipt: Receipt = { status: receiptStatus };
