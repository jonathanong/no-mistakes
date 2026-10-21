import client from "@app/jobs";
import { enqueueEmail } from "@app/jobs";

client.enqueueEmail("1");
const send = enqueueEmail;
send("1");
enqueueEmail.call(null, "1");
(0, enqueueEmail)("1");
client[`enqueueEmail`]("1");
client?.enqueueEmail("1");
