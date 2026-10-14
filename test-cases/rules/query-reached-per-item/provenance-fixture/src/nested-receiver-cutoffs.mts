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
