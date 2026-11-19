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
const object = { addAndWait: 1 };
queue.addAndWait();
