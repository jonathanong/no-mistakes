const client = { responses: {} };
function otherApi(input) { return `other:${input}`; }
// The selected property is absent, so this invokes another API's fallback.
const { create: fallback = otherApi } = client.responses;
fallback('input');
