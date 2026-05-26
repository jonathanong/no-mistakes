let sharedState = {};

function sharedCallback() {
  sharedState.value = 1;
}

{
  const sharedCallback = 0;
  it("does not resolve shadowed non-function callbacks", sharedCallback);
}
