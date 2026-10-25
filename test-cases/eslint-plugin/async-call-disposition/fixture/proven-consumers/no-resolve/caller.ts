import { enqueueEmail } from "@app/jobs";
import { processReconciliationPage } from "../cross-file-consumers";
const callback = () => enqueueEmail(1);
await processReconciliationPage(callback);
