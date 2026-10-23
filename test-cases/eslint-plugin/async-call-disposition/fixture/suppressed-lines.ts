import { enqueueEmail } from "@app/jobs";
// no-mistakes-disable-next-line async-call-disposition -- intentional detached job
enqueueEmail("1");
enqueueEmail("2"); // no-mistakes-disable-line no-mistakes/async-call-disposition -- detached
// no-mistakes-disable-next-line other-rule -- must not suppress this rule
enqueueEmail("3");
const text = "no-mistakes-disable-file async-call-disposition";
enqueueEmail("4");
