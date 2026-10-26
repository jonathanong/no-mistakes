import { enqueueEmail } from "@app/jobs";
async function consume(callback) { await callback(); }
const accepted = () => enqueueEmail();
await consume(accepted);
