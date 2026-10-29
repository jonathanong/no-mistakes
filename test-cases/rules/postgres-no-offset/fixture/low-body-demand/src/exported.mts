import { read } from '@example/db';
declare function invoke(callback: () => void): void;
export function provider() { invoke(() => { read('SELECT 1 OFFSET 6'); }); }
