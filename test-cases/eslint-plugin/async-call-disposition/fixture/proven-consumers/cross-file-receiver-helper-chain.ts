import { enqueueEmail } from "@app/jobs";
import { consume } from "./cross-file-receiver-consumers";

const twoLevel = () => enqueueEmail(1);
const forward = () => invoke();
const invoke = () => twoLevel();
await consume({ job: twoLevel, other: forward });

const safeA = () => safeB();
const safeB = () => safeA();
const safeJob = () => enqueueEmail(2);
await consume({ job: safeJob, other: safeA });

const unsafeA = () => unsafeB();
const unsafeB = () => {
  unsafeJob();
  unsafeA();
};
const unsafeJob = () => enqueueEmail(3);
await consume({ job: unsafeJob, other: unsafeA });

const heldJob = () => enqueueEmail(4);
const holder = { run: heldJob };
await consume({ job: heldJob, other: () => holder.run() });

const thisJob = () => enqueueEmail(5);
const readsThis = () => this.job();
await consume({ job: thisJob, other: readsThis });

const evalJob = () => enqueueEmail(6);
const usesEval = () => eval("job()");
await consume({ job: evalJob, other: usesEval });

const argumentsJob = () => enqueueEmail(7);
const readsArguments = () => void arguments;
await consume({ job: argumentsJob, other: readsArguments });

const unknownJob = () => enqueueEmail(8);
await consume({ job: unknownJob, other: () => unknownHelper() });
declare function unknownHelper(): void;

const nestedHolderJob = () => enqueueEmail(9);
const nestedHolder = { inside: { run: nestedHolderJob } };
await consume({ job: nestedHolderJob, other: () => nestedHolder.inside.run() });

// Deep pure helpers exceed the bounded proof; keep the diagnostic rather than guessing.
const deep0 = () => deep1();
const deep1 = () => deep2();
const deep2 = () => deep3();
const deep3 = () => deep4();
const deep4 = () => deep5();
const deep5 = () => deep6();
const deep6 = () => deep7();
const deep7 = () => deep8();
const deep8 = () => deep9();
const deep9 = () => deep10();
const deep10 = () => deep11();
const deep11 = () => deep12();
const deep12 = () => deep13();
const deep13 = () => deep14();
const deep14 = () => deep15();
const deep15 = () => deep16();
const deep16 = () => deep17();
const deep17 = () => deep18();
const deep18 = () => deep19();
const deep19 = () => deep20();
const deep20 = () => deep21();
const deep21 = () => deep22();
const deep22 = () => deep23();
const deep23 = () => deep24();
const deep24 = () => deep25();
const deep25 = () => deep26();
const deep26 = () => deep27();
const deep27 = () => deep28();
const deep28 = () => deep29();
const deep29 = () => deep30();
const deep30 = () => deep31();
const deep31 = () => deep32();
const deep32 = () => deep33();
const deep33 = () => 1;
const boundedJob = () => enqueueEmail(10);
await consume({ job: boundedJob, other: deep0 });
