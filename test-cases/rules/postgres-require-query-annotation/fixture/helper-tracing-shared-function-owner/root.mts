import { read } from '@bench/db';
function factory(text) { return () => text; }
const first = factory('SELECT 1');
const second = factory('/* named */ SELECT 2');
const alias = first;
read('SELECT 3');
read('/* named */ SELECT 4');
read(first());
read(second());
