import { primary } from '@vendor/valkey';
declare const opaque: unknown;
let unknownBox = { client: primary };
let primitiveBox = { client: primary };
function rebindUnknown() { unknownBox.client = opaque; }
function rebindPrimitive() { primitiveBox.client = 42; }
rebindUnknown();
rebindPrimitive();
// A provider summary must never invent a callable target for opaque/primitive RHS values.
unknownBox.client.get('opaque');
primitiveBox.client.get('primitive');
