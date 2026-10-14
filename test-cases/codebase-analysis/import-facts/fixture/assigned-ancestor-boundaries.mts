import { phased, ordinary, captured } from '@vendor/valkey';
declare const source: any;
declare let unused: any;
function read(value: any) { return value; }
({ first: phased.nested, later: unused = read(phased) } = read(phased));
read(phased);
read(ordinary);
ordinary.nested = {};
read(ordinary);
function writeCaptured() { captured.nested = {}; }
read(captured);
