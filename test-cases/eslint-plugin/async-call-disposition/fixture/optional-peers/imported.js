import { enqueueEmail } from "@app/jobs";
import { consume, consumeAgain, consumeProperties } from "./consumer.js";
// Distinct bindings and DI properties reuse one missing-peer result per context.
const first = () => enqueueEmail();
await consume(first);
const second = () => enqueueEmail();
await consumeAgain(second);
const property = () => enqueueEmail();
await consumeProperties({ enqueueEmail: property });
