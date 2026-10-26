// Production use of one symbol must not prove use of another export.
export function productionUsed() { return 1; }
export function noImporters() { return 2; }
export default function defaultUsed() { return 3; }
export interface Contract { value: number }
