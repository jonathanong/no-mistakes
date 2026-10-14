// Only a guaranteed fresh overwrite can sever the original receiver path.
function replacement() {}
const items = ['one'];

import { primary as client0 } from '@vendor/valkey';
{
  const client = client0;
  const box = { client };
  if (flag) box.client = { get: replacement };
  box.client.get = replacement;
  function conditional(value) { value.get('key'); }
  for (const item of items) { conditional(client0); }
}

import { primary as client1 } from '@vendor/valkey';
{
  const client = client1;
  const box = { client };
  flag && (box.client = { get: replacement });
  box.client.get = replacement;
  function shortCircuit(value) { value.get('key'); }
  for (const item of items) { shortCircuit(client1); }
}

import { primary as client2 } from '@vendor/valkey';
{
  const client = client2;
  const box = { client };
  box.client ||= { get: replacement };
  box.client.get = replacement;
  function logicalAssignment(value) { value.get('key'); }
  for (const item of items) { logicalAssignment(client2); }
}

import { primary as client3 } from '@vendor/valkey';
{
  const client = client3;
  const box = { client };
  for (const item of maybeEmpty) { box.client = { get: replacement }; }
  box.client.get = replacement;
  function loop(value) { value.get('key'); }
  for (const item of items) { loop(client3); }
}

import { primary as client4 } from '@vendor/valkey';
{
  const client = client4;
  const box = { client };
  const { value = (box.client = { get: replacement }) } = source;
  box.client.get = replacement;
  function bindingDefault(value) { value.get('key'); }
  for (const item of items) { bindingDefault(client4); }
}

import { primary as client5 } from '@vendor/valkey';
{
  const client = client5;
  const box = { client };
  ({ value: target = (box.client = { get: replacement }) } = source);
  box.client.get = replacement;
  function assignmentDefault(value) { value.get('key'); }
  for (const item of items) { assignmentDefault(client5); }
}

import { primary as client6 } from '@vendor/valkey';
{
  const client = client6;
  const box = { client };
  box.client = client;
  box.client.get = replacement;
  function sameReceiver(value) { value.get('key'); }
  for (const item of items) { sameReceiver(client6); }
}

import { primary as client7 } from '@vendor/valkey';
{
  const client = client7;
  const box = { client };
  box.client = unknown;
  box.client.get = replacement;
  function unknownReceiver(value) { value.get('key'); }
  for (const item of items) { unknownReceiver(client7); }
}

import { primary as client8 } from '@vendor/valkey';
{
  const client = client8;
  const box = { client };
  if (flag) box.client = { get: replacement };
  box.client = { get: replacement };
  box.client.get = replacement;
  function guaranteedAfterConditional(value) { value.get('key'); }
  for (const item of items) { guaranteedAfterConditional(client8); }
}

import { primary as parameterClient } from '@vendor/valkey';
const parameterBox = { client: parameterClient };
function parameterWrite(value = (parameterBox.client = { get: replacement })) { parameterBox.client.get = replacement; }
parameterWrite(supplied);
function parameterDefault(value) { value.get('key'); }
for (const item of items) { parameterDefault(parameterClient); }

// Acquisition before the guaranteed overwrite keeps the earlier receiver.
import { primary as acquiredClient } from '@vendor/valkey';
const acquiredBox = { client: acquiredClient };
if (flag) acquiredBox.client = { get: replacement };
const acquired = acquiredBox.client;
acquiredBox.client = { get: replacement };
acquired.get = replacement;
function acquiredBeforeGuaranteed(value) { value.get('key'); }
for (const item of items) { acquiredBeforeGuaranteed(acquiredClient); }

// A loop exit can bypass an otherwise unconditional-looking body assignment.
import { primary as forClient } from '@vendor/valkey';
const forBox = { client: forClient };
for (;;) { break; forBox.client = { get: replacement }; }
forBox.client.get = replacement;
function forBody(value) { value.get('key'); }
for (const item of items) { forBody(forClient); }

import { primary as whileClient } from '@vendor/valkey';
const whileBox = { client: whileClient };
while (true) { break; whileBox.client = { get: replacement }; }
whileBox.client.get = replacement;
function whileBody(value) { value.get('key'); }
for (const item of items) { whileBody(whileClient); }

import { primary as doClient } from '@vendor/valkey';
const doBox = { client: doClient };
do { break; doBox.client = { get: replacement }; } while (false);
doBox.client.get = replacement;
function doBody(value) { value.get('key'); }
for (const item of items) { doBody(doClient); }

import { primary as client14 } from '@vendor/valkey';
const arrayBox = { client: client14 };
// Array binding defaults are conditional just like object binding defaults.
const [arrayValue = (arrayBox.client = { get: replacement })] = source;
arrayBox.client.get = replacement;
function arrayBindingDefault(value) { value.get('key'); }
for (const item of items) { arrayBindingDefault(client14); }

import { primary as client15 } from '@vendor/valkey';
const tryBox = { client: client15 };
// A caught throw can bypass the fresh reference replacement.
try { mayThrow(); tryBox.client = {}; } catch {}
tryBox.client.get = replacement;
function caughtReplacement(value) { value.get('key'); }
for (const item of items) { caughtReplacement(client15); }

import { primary as client16 } from '@vendor/valkey';
const restoredBox = { client: client16 };
restoredBox.client = {};
restoredBox.client = client16;
restoredBox.client.get = replacement;
function restoredReference(value) { value.get('key'); }
for (const item of items) { restoredReference(client16); }

import { primary as client17 } from '@vendor/valkey';
const dynamicBox = { client: client17 };
dynamicBox.client = {};
dynamicBox[key] = client17;
dynamicBox.client.get = replacement;
function dynamicRestoration(value) { value.get('key'); }
for (const item of items) { dynamicRestoration(client17); }

import { primary as client18 } from '@vendor/valkey';
const savedFreshBox = { client: client18 };
savedFreshBox.client = {};
const savedFresh = savedFreshBox.client;
// Restoring the live container cannot change this previously acquired receiver.
savedFreshBox.client = client18;
savedFresh.get = replacement;
function savedBeforeRestoration(value) { value.get('key'); }
for (const item of items) { savedBeforeRestoration(client18); }

import { primary as client19 } from '@vendor/valkey';
const savedDynamicBox = { client: client19 };
savedDynamicBox.client = {};
const savedDynamic = savedDynamicBox.client;
savedDynamicBox[key] = client19;
savedDynamic.get = replacement;
function savedBeforeDynamicRestoration(value) { value.get('key'); }
for (const item of items) { savedBeforeDynamicRestoration(client19); }

import { primary as client20 } from '@vendor/valkey';
const cycleBox = { client: client20 };
// The live first branch mutates primary; the fallback graph also contains a cycle.
const { client: cycle1 = cycle2.client } = cycleBox;
const { client: cycle2 = cycle1.client } = cycle1;
cycle1.get = replacement;
function cyclicAcquisition(value) { value.get('key'); }
for (const item of items) { cyclicAcquisition(client20); }

import { primary as client21 } from '@vendor/valkey';
const labelBox = { client: client21 };
outer: { if (flag) break outer; labelBox.client = {}; }
labelBox.client.get = replacement;
function labeledBypass(value) { value.get('key'); }
for (const item of items) { labeledBypass(client21); }

import { primary as client22 } from '@vendor/valkey';
const plainFreshBox = { client: client22 };
plainFreshBox.client = {};
plainFreshBox.client.get = replacement;
function unlabeledFresh(value) { value.get('key'); }
for (const item of items) { unlabeledFresh(client22); }

import { primary as client23 } from '@vendor/valkey';
const logicalFreshBox = { client: client23 };
logicalFreshBox.client = {};
logicalFreshBox.client &&= {};
logicalFreshBox.client ||= {};
logicalFreshBox.client ??= {};
logicalFreshBox.client.get = replacement;
function logicalFreshPreserved(value) { value.get('key'); }
for (const item of items) { logicalFreshPreserved(client23); }

import { primary as client24 } from '@vendor/valkey';
const logicalAndBox = { client: client24 };
logicalAndBox.client &&= {};
logicalAndBox.client.get = replacement;
function logicalAndUncertain(value) { value.get('key'); }
for (const item of items) { logicalAndUncertain(client24); }

import { primary as client25 } from '@vendor/valkey';
const logicalOrBox = { client: client25 };
logicalOrBox.client ||= {};
logicalOrBox.client.get = replacement;
function logicalOrUncertain(value) { value.get('key'); }
for (const item of items) { logicalOrUncertain(client25); }

import { primary as client26 } from '@vendor/valkey';
const logicalNullBox = { client: client26 };
logicalNullBox.client ??= {};
logicalNullBox.client.get = replacement;
function logicalNullUncertain(value) { value.get('key'); }
for (const item of items) { logicalNullUncertain(client26); }

import { primary as client27 } from '@vendor/valkey';
const compoundBox = { client: client27 };
compoundBox.client = {};
// Unsupported compound RHS evaluation cannot establish replacement identity.
compoundBox.client += {};
compoundBox.client.get = replacement;
function compoundUncertain(value) { value.get('key'); }
for (const item of items) { compoundUncertain(client27); }
