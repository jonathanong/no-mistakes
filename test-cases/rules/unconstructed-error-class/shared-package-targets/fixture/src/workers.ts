import { WorkerEntrypoint } from 'cloudflare:workers';
export class ApplicationWorker extends WorkerEntrypoint<{ value: string }> {}
