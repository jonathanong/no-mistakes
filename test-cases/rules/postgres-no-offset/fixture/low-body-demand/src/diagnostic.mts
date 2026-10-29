import { read } from '@example/db';
declare function invoke(callback: () => void): void;
const duplicate = 1;
const duplicate = 2;
invoke(() => { read('SELECT 1 OFFSET 7'); });
