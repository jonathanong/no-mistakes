import { fetch as importedFetch } from "undici";

function withParameter(fetch: (path: string) => Promise<Response>) {
  return fetch("/api/local");
}

const fetch = createFetch();
fetch("/api/local-binding");

importedFetch("/api/imported");
client.fetch("/api/client");

const window = { fetch: createFetch() };
window.fetch("/api/window-shadow");

const globalThis = { fetch: createFetch() };
globalThis.fetch("/api/global-shadow");

let alias = self.fetch;
alias("/api/mutable-alias");

const dynamic = self[name];
dynamic("/api/dynamic");

// These destructures are intentionally unsupported as fetch aliases.
const { otherFetch, ...restFetchGlobals } = self;
const { fetch: {} } = globalThis;
