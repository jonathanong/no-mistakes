import { transaction } from "@sample/database";
import { enqueue } from "@sample/jobs";

await transaction("write", () => enqueue({ receiptId: "excluded" }));
