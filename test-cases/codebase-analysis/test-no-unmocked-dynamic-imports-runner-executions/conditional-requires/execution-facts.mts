// Literal built-in specifiers resolve; the nonliteral require below deliberately
// probes computed-import facts.
require('node:fs');
true && require('node:path');
false || require('node:assert');
null ?? require('node:buffer');
null || require('node:module');
if (0) require('node:os'); else require('node:crypto');
if ('yes') require('node:url'); else require('node:os');
true ? require('node:util') : require('node:os');
false ? require('node:os') : require('node:stream');
if (globalThis.flag) require('node:os'); else require('node:os');
globalThis.flag && require('node:os');
globalThis.flag || require('node:os');
globalThis.flag ?? require('node:os');
globalThis.flag ? require('node:os') : require('node:os');
globalThis.flag &&= require('node:os');
globalThis.flag ||= require('node:os');
globalThis.flag ??= require('node:os');
globalThis.flag = require('node:dns');
while (false) require('node:os');
while (true) { require('node:events'); break; }
for (require('node:zlib'); false; require('node:os')) require('node:os');
for (; true; require('node:os')) { require('node:net'); break; }
for (;;) { require('node:tls'); break; }
for (const key in require('node:http')) require('node:os');
for (const value of require('node:https')) require('node:os');
try { require('node:querystring'); } catch (error) { require('node:os'); }
switch (require('node:punycode')) {
  case require('node:os'): require('node:os'); break;
  default: require('node:os');
}
do { require('node:process'); } while (false);
function unused() { require('node:os'); }
false && require('node:os');
true || require('node:os');
1 ?? require('node:os');
require?.('node:os');
globalThis.maybe?.(require('node:os'));
globalThis.maybe?.method(require('node:os'));
globalThis.maybe?.().method(require('node:os'));
globalThis.maybe?.[require('node:os')];
globalThis.maybe?.nested[require('node:os')];
require('node:tty')?.method(require('node:os'));
globalThis[require('node:readline')];
require('node:vm')?.(require('node:os'));
globalThis.maybe<unknown>(require('node:worker_threads'));
0 && require('node:os');
1 && require('node:constants');
'' || require('node:domain');
'' && require('node:os');
if (false) { if (true) require('node:os'); }
require.resolve('node:os');
import type { Platform } from 'node:os';
import('node:os');
require(globalThis.target);
require();
require('node:async_hooks');
