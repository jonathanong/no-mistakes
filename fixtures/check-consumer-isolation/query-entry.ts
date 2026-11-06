import { load } from './broken';
const items = [1, 2];
export function run() { return items.map(item => load(item)); }
