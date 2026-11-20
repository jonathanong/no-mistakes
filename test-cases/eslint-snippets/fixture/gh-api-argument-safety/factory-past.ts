declare function makeClient(options: unknown): { call(args: unknown[]): void };
const exec = {};
makeClient({ exec }).call(["repos/EXAMPLE/DEMO/labels?per_page=1"]);
