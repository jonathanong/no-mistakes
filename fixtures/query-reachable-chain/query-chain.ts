import { load } from './broken';
function leaf(value: number) { return load(value); }
function helper(value: number) { return leaf(value); }
export function run() { return [1, 2].map(item => helper(item)); }
