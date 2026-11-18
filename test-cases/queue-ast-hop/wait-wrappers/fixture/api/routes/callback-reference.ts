declare const client: any;
declare function run(callback: () => void): void;
run(client.enqueueCrawlUrlAndWait);
