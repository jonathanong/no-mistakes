import { createQueue, enqueueProperty, makeEnqueue } from "./wrappers";
import type { makeEnqueue as TypeFactory } from "./wrappers";
import { type makeEnqueue as TypedSpecifier } from "./wrappers";
import * as namespaceFactory from "./wrappers";

declare const unknownEnvelope: Record<string, unknown>;
declare const unknownOptions: unknown;
const queue = createQueue("wrapper-unit");

enqueueProperty(queue, "aliased", { options: { attempts: 2, removeOnComplete: true, priority: 3 } });
enqueueProperty(queue, "aliased", unknownEnvelope);
enqueueProperty(queue, "aliased", { options: 42 });
enqueueProperty(queue, "aliased", { options: { attempts: 2 }, ...unknownEnvelope });
enqueueProperty(queue, "aliased", { options: unknownOptions });
enqueueProperty(queue, "aliased", { options: { attempts: 2, removeOnComplete: true, priority: 3, deduplication: 42 } });
enqueueProperty(queue, "aliased", 42);
const direct = enqueueProperty(queue, "aliased", { options: { attempts: 2, removeOnComplete: true, priority: 3 } });
direct({}, { jobId: "bad:second-call" });

function unrelatedFactory() { return (_data: unknown) => {}; }
const unrelated = unrelatedFactory();
unrelated({});

const typeOnly = TypeFactory({ name: "typeOnly" });
typeOnly({});
const typedSpecifier = TypedSpecifier({ name: "typedSpecifier" });
typedSpecifier({});
const namespace = namespaceFactory.makeEnqueue({ name: "namespace" });
namespace({});
enqueueProperty({}, "aliased", { options: { attempts: 2, removeOnComplete: true, priority: 3 } });

const computed = makeEnqueue({ ["name"]: "computed" }, {});
computed({});
const getter = makeEnqueue({ get name() { return "getter"; } }, {});
getter({});
const proto = makeEnqueue({ __proto__: {}, name: "proto" }, {});
proto({});

const computedWriteDef = { name: "beforeComputedWrite" };
computedWriteDef["name"] = "afterComputedWrite";
const computedWrite = makeEnqueue(computedWriteDef, {});
computedWrite({});

const computedDeleteDef = { name: "beforeComputedDelete" };
delete computedDeleteDef["name"];
const computedDelete = makeEnqueue(computedDeleteDef, {});
computedDelete({});

const nestedDef = { name: "beforeNestedWrite", metadata: { value: 1 } };
const nested = nestedDef.metadata;
nested.value = 2;
const nestedWrite = makeEnqueue(nestedDef, {});
nestedWrite({});

const computedNestedDef = { name: "beforeComputedNestedWrite", metadata: { value: 1 } };
const computedNested = computedNestedDef["metadata"];
computedNested.value = 2;
const computedNestedWrite = makeEnqueue(computedNestedDef, {});
computedNestedWrite({});

const forward = makeEnqueue(laterDefinition, {});
forward({});
const laterDefinition = { name: "forward" };

const optionsWithSpread = makeEnqueue({ name: "aliased" }, {});
optionsWithSpread({}, { ...unknownEnvelope });
const missingProperty = enqueueProperty(queue, "aliased", { other: "unresolved" }, { attempts: 2, removeOnComplete: true, priority: 3 });
missingProperty({});

import { DESTRUCTURED } from "./definitions";
const destructured = makeEnqueue(DESTRUCTURED, {});
destructured({});

declare const unknownArguments: [object, string, unknown];
enqueueProperty(...unknownArguments);
let unrelatedCount = 0;
unrelatedCount += 1;
void unrelatedCount;
