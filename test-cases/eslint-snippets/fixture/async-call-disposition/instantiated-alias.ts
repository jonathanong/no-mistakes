import { enqueueEmail } from "@app/jobs";

type Job = { id: string };

const send = enqueueEmail<Job>;
send({ id: "1" });

const nested = (0, enqueueEmail<Job>);
nested({ id: "2" });
