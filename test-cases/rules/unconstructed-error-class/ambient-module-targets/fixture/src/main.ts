import { WorkerEntrypoint, ExternalError } from 'fixture:runtime';
export class ApplicationWorker extends WorkerEntrypoint<{ value: string }, unknown> {}
export class UnusedError extends ExternalError {}
export class UsedError extends ExternalError {}
new UsedError();
