import { primary } from '@vendor/valkey';
export function make() { return primary.get; }
