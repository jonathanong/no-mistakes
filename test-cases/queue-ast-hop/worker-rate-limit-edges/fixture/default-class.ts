import { Worker } from "bullmq";

export default class NamedDefaultWorker extends Worker {}
export class SubOfNamedDefaultWorker extends NamedDefaultWorker {}
