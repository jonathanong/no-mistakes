// no-mistakes-disable-file async-call-disposition -- intentional detached jobs
import { enqueueEmail } from "@app/jobs";
enqueueEmail("1");
