declare const dependencies: any;
declare const queue: any;
declare const job: any;
const { enqueueReviewAndWait: enqueueReview } = dependencies;
const { addAndWait: waitForJob, add } = queue;
const { waitUntilFinished } = job;
const { outer: { addAndWait: nested } } = queue;
const { addAndWait: withDefault = fallback, enqueueReviewAndWait = fallback } = queue;
const { "addAndWait": quoted } = queue;
const { [dynamicKey]: computed, ...rest } = queue;
function handler({ waitUntilFinished: inParameter }: any) {}
declare const batches: readonly unknown[];
let addAndWait: unknown;
let assignedReview: unknown;
let assignedRest: unknown;
for (const { addAndWait: fromLoop, kept } of batches) {}
for (const { outer: { waitUntilFinished: fromNestedLoop } } of batches) {}
({ addAndWait, ...assignedRest } = queue);
({ enqueueReviewAndWait: assignedReview } = dependencies);
const { ['addAndWait']: literalComputed, ['kept']: keptComputed } = queue;
const object = { addAndWait: 1 };
queue.addAndWait();
