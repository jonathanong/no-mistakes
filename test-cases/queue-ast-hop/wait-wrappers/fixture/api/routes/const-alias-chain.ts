declare const client: any;
const submit = client.enqueueCrawlUrlAndWait;
const retry = submit;
retry();
