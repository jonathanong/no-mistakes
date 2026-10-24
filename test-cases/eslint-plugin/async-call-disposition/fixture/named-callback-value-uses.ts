import { enqueueEmail } from "@app/jobs";
import { awaitReportedEnqueueAdmission as consume } from "@app/consumer";

const typeofOnly = () => enqueueEmail(1);
console.log(typeof typeofOnly);
const binaryOnly = () => enqueueEmail(2);
assert(binaryOnly !== undefined);
const unaryOnly = () => enqueueEmail(3);
console.log(!unaryOnly);
let updateOnly = () => enqueueEmail(4);
console.log(updateOnly++);
const templateOnly = () => enqueueEmail(5);
console.log(`${templateOnly}`);
const conditionalTest = () => enqueueEmail(6);
unknownConsumer(conditionalTest ? other : fallback);
const sequenceFirst = () => enqueueEmail(7);
unknownConsumer((sequenceFirst, fallback));
let assignmentLeft = () => enqueueEmail(8);
unknownConsumer((assignmentLeft = fallback));
const assignmentKey = () => enqueueEmail(9);
unknownConsumer((bucket[assignmentKey] = fallback));

// A tag receives raw interpolation values; these branches can still be callbacks.
const tagged = () => enqueueEmail(10);
tag`${tagged}`;
const conditionalValue = () => enqueueEmail(11);
unknownConsumer(flag ? conditionalValue : fallback);
const logicalValue = () => enqueueEmail(12);
consume(flag && logicalValue);
const sequenceLast = () => enqueueEmail(13);
consume((fallback, sequenceLast));
const assignmentRight = () => enqueueEmail(14);
let holder;
unknownConsumer((holder = assignmentRight));
const directUnknown = () => enqueueEmail(15);
unknownConsumer(directUnknown);
const directKnown = () => enqueueEmail(16);
consume(directKnown);
let total = 0;
const compoundOnly = () => enqueueEmail(17);
unknownConsumer((total += compoundOnly));
// Calling a function as a tag or constructor does not pass the function itself.
const calledTag = () => enqueueEmail(18);
unknownConsumer(calledTag`text`);
function calledConstructor() { return enqueueEmail(19); }
unknownConsumer(new calledConstructor());
// Computed property and method keys coerce the function to a property key.
const objectKey = () => enqueueEmail(20);
unknownConsumer({ [objectKey]: 1, [objectKey]() {} });
const classKey = () => enqueueEmail(21);
unknownConsumer(class { [classKey]() {} });
// An erased type argument never passes the runtime function as a callback.
const erasedReference = () => enqueueEmail(22);
unknownConsumer<typeof erasedReference>();
const constructedArgument = () => enqueueEmail(23);
unknownConsumer(new Holder(constructedArgument));
const computedValue = () => enqueueEmail(24);
unknownConsumer({ [flag]: computedValue });
const assertedBranch = () => enqueueEmail(25);
consume((flag ? assertedBranch : fallback) as unknown);
