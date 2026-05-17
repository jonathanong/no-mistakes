const handler = () => {};
const ready = true;

app.route("/chain").get(handler).post(handler);
app.put("/direct", handler);
wrap(app.patch("/wrapped", handler), other("/ignored"));

{
  app.delete("/block", handler);
}

function nested() {
  if (ready) {
    app.head("/if", handler);
  } else {
    app.options("/else", handler);
  }
  while (ready) {
    break;
  }
  do {
    break;
  } while (ready);
  switch (ready) {
    case true:
      app.get("/switch", handler);
      break;
    default:
      app.get("/default", handler);
  }
  try {
    app.get("/try", handler);
  } catch (error) {
    app.get("/catch", handler);
  } finally {
    app.get("/finally", handler);
  }
}

export const exportedRoute = app.get("/export-var", handler);

export function exportedFunction() {
  app.get("/export-function", handler);
}

function shadowedBlocks() {
  const app = fake;
  app.get("/ignored-const", handler);
}

function shadowedPatterns() {
  const { app: alias, other } = fake;
  const [app] = fake;
  const [first, ...rest] = fake;
  const { nested: { app: nestedApp }, ...others } = fake;
  alias.get("/ignored-alias", handler);
  nestedApp.get("/ignored-nested", handler);
}

function shadowedVarInControlFlow() {
  if (ready) {
    var app = fake;
  }
  app.get("/ignored-var-if", handler);
}

function shadowedForIn() {
  for (var app in apps) {
  }
  app.get("/ignored-for-in", handler);
}

function shadowedForOf() {
  for (var app of apps) {
  }
  app.get("/ignored-for-of", handler);
}

function app() {
  app.get("/ignored-function-name", handler);
}

