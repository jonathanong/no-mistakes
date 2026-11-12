function blocks() { { { expression(); } } }
function loop() { for (const value of values) { const item = value; expression(item); } }
function directLoop() { for (const value of values) expression(value); }
function returnedPromise() { return operation(); }
function loopReturnedPromise() { for (const value of values) { return operation(value); } }
async function awaitedLoop() { for await (const value of values) { expression(value); } }
function depthAccepted() { { { { { { { { { { { { { { { { expression(); } } } } } } } } } } } } } } } }
function depthRejected() { { { { { { { { { { { { { { { { { expression(); } } } } } } } } } } } } } } } } }
