import {read} from '@bench/db';
import * as safe from './producer.mjs';
const {named} = safe;
read(named()); // low: true
import type * as typed from './producer.mjs';
const {named: typedHelper} = typed;
read(typedHelper()); // low: false
import * as nested from './producer.mjs';
function nestedScope() { const {named: nestedHelper} = nested; read(nestedHelper()); } // low: false
import * as exported from './producer.mjs';
export const {named: exportedHelper} = exported;
read(exportedHelper()); // low: false
import * as typeUse from './producer.mjs';
const {named: withType} = typeUse;
type Factory = typeof withType;
read(withType()); // low: false
