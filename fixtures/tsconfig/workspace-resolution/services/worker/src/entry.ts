import { runtime } from "@runtime/value";
import { message } from "@shared/message";

export const workerEntry = `${runtime}:${message}`;
