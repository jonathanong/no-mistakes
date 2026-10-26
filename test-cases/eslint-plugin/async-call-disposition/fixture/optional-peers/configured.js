import { enqueueEmail } from "@app/jobs";
import { consume, consumeProperties } from "./consumer.js";
await consume(() => enqueueEmail());
await consumeProperties({ enqueueEmail: () => enqueueEmail() });
// Named callbacks must honor contracts before optional imported-body proof.
const named = () => enqueueEmail();
await consume(named);
const property = () => enqueueEmail();
await consumeProperties({ enqueueEmail: property });
