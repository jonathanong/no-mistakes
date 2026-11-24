import { Worker } from "bullmq";

export class ExportedWorker extends Worker {}

export class SubExportedWorker extends ExportedWorker {}

export const exportedSubInstance = new ExportedWorker();
