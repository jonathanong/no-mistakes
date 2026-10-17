import { primary } from '@vendor/valkey';
function consume(value) { value.get('stable'); }
consume(primary);
