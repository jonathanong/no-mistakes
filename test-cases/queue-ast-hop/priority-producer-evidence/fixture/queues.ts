import { Queue, Worker } from "bullmq";

export const sameLine = new Queue("sameLine");
export const numeric = new Queue("numeric");
export const computed = new Queue("computed");
export const unknownComputed = new Queue("unknownComputed");
export const duplicate = new Queue("duplicate");
export const unknownBulkName = new Queue("unknownBulkName");
export const shadowed = new Queue("shadowed");
export const crossFileFlow = new Queue("crossFileFlow");
export const ambiguousFlow = new Queue("ambiguousFlow");
export const staticBlock = new Queue("staticBlock");
export const sameCase = new Queue("sameCase");

new Worker("sameLine", async () => {});
new Worker("numeric", async () => {});
new Worker("computed", async () => {});
new Worker("unknownComputed", async () => {});
new Worker("duplicate", async () => {});
new Worker("unknownBulkName", async () => {});
new Worker("shadowed", async () => {});
new Worker("crossFileFlow", async () => {});
new Worker("ambiguousFlow", async () => {});
new Worker("staticBlock", async () => {});
new Worker("sameCase", async () => {});
