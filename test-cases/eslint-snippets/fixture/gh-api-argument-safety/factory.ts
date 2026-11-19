declare function makeClient(options: unknown): { call(args: unknown[]): void; other(args: unknown[]): void };
declare function other(): { call(args: unknown[]): void };
const exec = {};
makeClient({ exec }).call(['repos/EXAMPLE/DEMO/labels?per_page=1']);
makeClient({ exec }).call(['repos/EXAMPLE/DEMO/labels', '-F', 'per_page=1']);
makeClient({ exec }).call(['-X', 'GET', 'repos/EXAMPLE/DEMO/labels', '-F', 'per_page=1']);
other().call(['repos/EXAMPLE/DEMO/labels?per_page=1']);
makeClient({ exec }).other(['repos/EXAMPLE/DEMO/labels?per_page=1']);
makeClient({ exec })['call'](['repos/EXAMPLE/DEMO/labels?per_page=1']);
makeClient({ exec })[key](['repos/EXAMPLE/DEMO/labels?per_page=1']);
makeClient({ exec }).call(['api', 'repos/EXAMPLE/DEMO/labels?per_page=1']);
function shadow(makeClient: typeof other) { makeClient().call(['repos/EXAMPLE/DEMO/labels?per_page=1']); }
const client = makeClient({ exec });
client.call(['repos/EXAMPLE/DEMO/labels?per_page=1']);
(0, makeClient)({ exec }).call(['repos/EXAMPLE/DEMO/labels?per_page=1']);
