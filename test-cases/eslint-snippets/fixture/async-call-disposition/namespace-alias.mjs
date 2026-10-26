import * as jobs from "@app/jobs";

const api = jobs;
api.enqueueEmail("1");

const client = api;
client.enqueueEmail("2");

let rebound = jobs;
rebound.enqueueEmail("3");
