let sharedState = [];

function runTest() {
  const sharedState = [];
  sharedState.push("local");
}

it("does not report module state for local shadow in callback", runTest);
