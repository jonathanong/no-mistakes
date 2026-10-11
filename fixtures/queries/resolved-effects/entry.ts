import { write as save, Client } from "@vendor/effects";
import * as effects from "@vendor/effects";
save(1);
effects.write(2);
const alias = save;
alias(3);
const memberAlias = effects.write;
memberAlias(4);
new Client();
function write() {}
write();
function shadow(save: () => void) { save(); }
function shadowNamespace(effects: { write(): void }) { effects.write(); }
import { write as other } from "@other/effects";
other();
export function execute() { save(5); }
import { forwarded } from './external-barrel';
forwarded(6);
import { write as local } from './local';
local();
import { write as viaLocalBarrel } from './local-barrel';
viaLocalBarrel();
