import { receiver } from './receiver.mts';
export function acquire() { return receiver.get; }
