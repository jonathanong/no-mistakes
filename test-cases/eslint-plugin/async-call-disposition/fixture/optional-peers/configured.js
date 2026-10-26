import { enqueueEmail } from "@app/jobs";
import { consume, consumeProperties } from "./consumer.js";
await consume(() => enqueueEmail());
await consumeProperties({ enqueueEmail: () => enqueueEmail() });
