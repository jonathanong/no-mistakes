// no-mistakes-disable-file queue-no-enqueue-in-transaction -- This module owns a sole durable handoff.
import { transaction } from "@sample/database";
import { enqueue } from "@sample/jobs";

await transaction("write", () => enqueue({ receiptId: "file" }));
