import { load } from './barrel';
const items = [1, 2];
export function run() { return items.map(item => load(item)); }
