import { primary } from '@vendor/valkey';
export function factory() { return ({ client = primary } = {}) => client.get(); }
