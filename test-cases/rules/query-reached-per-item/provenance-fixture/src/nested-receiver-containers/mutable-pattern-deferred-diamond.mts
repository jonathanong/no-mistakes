import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
let box = { client };
const sink = {};
declare const key: string;
const d0 = { box };
const d1 = { left: d0, right: d0 };
const d2 = { left: d1, right: d1 };
const d3 = { left: d2, right: d2 };
const d4 = { left: d3, right: d3 };
const d5 = { left: d4, right: d4 };
const d6 = { left: d5, right: d5 };
const d7 = { left: d6, right: d6 };
const d8 = { left: d7, right: d7 };
const d9 = { left: d8, right: d8 };
const d10 = { left: d9, right: d9 };
const d11 = { left: d10, right: d10 };
const d12 = { left: d11, right: d11 };
const d13 = { left: d12, right: d12 };
const d14 = { left: d13, right: d13 };
const d15 = { left: d14, right: d14 };
const d16 = { left: d15, right: d15 };
// One physical write fans through shared aliases; replay must retain DAG sharing.
[sink[(d16[key][key][key][key][key][key][key][key][key][key][key][key][key][key][key][key].box.client.get = replacement, 'slot')]] = [(box = { client: replica }, 0)];
