import { primary } from '@vendor/valkey';
// A mutable scalar alias does not become a tracked receiver origin.
const seed = 1;
let scalar = seed;
scalar++;
function consume(value) { value.get('stable'); }
consume(primary);
