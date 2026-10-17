import { receiver } from '@vendor/foreign-parameter-many';
export function acquire(method = receiver.get) { return method; }
