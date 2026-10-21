import client, { enqueueEmail } from "@app/jobs";

const send = (0, enqueueEmail);
send("1");

const nested = (0, (0, enqueueEmail));
nested("2");

const member = (0, client.enqueueEmail);
member("3");

const other = (enqueueEmail, console.log);
other("4");
