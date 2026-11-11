import { events } from "./logger.mjs";
export function captureException() { events.push("captureException"); }
