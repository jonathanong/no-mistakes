import * as database from "@sample/database";
import { enqueue } from "@sample/jobs";

await database.transaction("write", () => enqueue({ receiptId: "moved" }));
