import express from "express";

const app = express();
const router = express.Router();

app.get("/api/v1/users", listUsers);
app.get("/api/v1/users/:id", getUser);
app.get("/api/v1/search", (req, res) => {
  const { term } = req.query;
  const page = req.query["page"];
  res.json({ term, page });
});
app.get("/api/v1/query-shapes", function (req, res) {
  // This fixture intentionally exercises static AST shapes for query-param
  // extraction; many branches are unreachable at runtime but parsable.
  const alias = req.query;
  let withoutInit;
  const { first: renamed, nested = "fallback" } = req.query;
  const { [dynamicKey]: computed = req.query.assignedPattern } = req.query;
  const assigned = (req.query.assigned = "value");
  const computedDynamic = req.query[dynamicKey];
  const call = req.query("call");
  const calls = req.queries("calls");
  const url = new URLSearchParams("?url=value").get("url");
  const ignored = other.get("ignored");
  const object = { nested: req.query.object };
  const array = [req.query.array];

  if (req.query.conditional) {
    req.query.ifBranch;
  } else {
    req.query.elseBranch;
  }

  for (const item of [req.query.forOf]) {
    item;
  }
  for (const key in { forIn: req.query.forIn }) {
    key;
  }
  for (let i = Number(req.query.forInit); i < Number(req.query.forTest); i += 1) {
    req.query.forBody;
  }
  let j = 0;
  for (j = Number(req.query.forExprInit); j < Number(req.query.forExprTest); j += 1) {
    req.query.forExprBody;
  }
  while (req.query.whileLoop) {
    break;
  }
  switch (req.query.switchOn) {
    case "a":
      req.query.switchCase;
      break;
  }
  try {
    req.query.tryBlock;
  } catch {
    req.query.catchBlock;
  } finally {
    req.query.finallyBlock;
  }
  try {
    req.query.tryWithoutCatch;
  } finally {
    req.query.finallyWithoutCatch;
  }
  try {
    req.query.tryWithoutFinally;
  } catch {
    req.query.catchWithoutFinally;
  }
  function nestedFunction() {
    return req.query.functionBody;
  }
  const nestedArrow = () => req.query.arrowBody;
  const nestedFunctionExpression = function () {
    return req.query.functionExpressionBody;
  };

  res.json({
    alias,
    withoutInit,
    renamed,
    nested,
    assigned,
    computed,
    computedDynamic,
    call,
    calls,
    url,
    ignored,
    object,
    array,
    nestedFunction,
    nestedArrow,
    nestedFunctionExpression,
  });
});
export class ExportedRouteClass {}
export declare function exportedDeclaredRoute(): void;
app.route(`/api/v1/users/:id`).patch(updateUser).delete(deleteUser);

router.post("/api/v1/users", createUser);

function listUsers() {}
function getUser() {}
function updateUser() {}
function deleteUser() {}
function createUser() {}
