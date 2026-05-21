import axios from "axios";
import got from "got";
import { request as undiciRequest } from "undici";
import { request as playwrightRequest } from "@playwright/test";
import { unused as nodeFetchUnused } from "node-fetch";
import {
  "get" as stringImportedGet,
  AxiosHeaders as ImportedHeaders,
  create as importedCreate,
  default as namedDefaultAxios,
  get as importedGet,
} from "axios";
import tsAxios = require("axios");

const { get, request } = axios;
const interopAxios = __importDefault(require("axios")).default;
const dynamicModule = "axios";
const ignoredDynamicRequire = require(dynamicModule);
const { create, AxiosHeaders } = axios;
const post = axios.post;
const { defaults } = axios;
const { ...restClient } = axios;
const { get: getWithDefault = axios.get } = axios;
const [ignoredArrayClient] = axios;
const optionalAlias = axios?.defaults;
const headers = require("axios").AxiosHeaders;
const cjsDefaultAxios = require("axios").default;
const { get: factoryGet } = axios.create();
const { localOnly, ...localRest } = {};
const [firstLocal, ...restLocalItems] = [];
const localSource = { get(_path: string) {} };
let assignedClient;
let assignedGet;
let assignedRequest;
let assignedCreate;
let assignedExtend;
let extend;
let ignoredAssignmentTarget;
let reassignedClient;
let blockReassignedClient;
let blockPromotedClient;
let logicalClient;
let nullishClient;
let destructuredGet;
let destructuredClient;
let put;
let shadowedDefaults;
let assignedArrayClient;
let assignedArrayRestClient;
let defaultedArrayClient;
let nestedDefaultGet;
let nestedObjectGet;
let nestedArrayClient;
let assignedGetShorthand;
let selfAssignedClient;
let response;
const holder = {};
const computedTarget = {};

get("/api/users");
request("/api/users");
create().get("/api/users");
AxiosHeaders.get("x");
importedCreate().get("/api/users");
importedGet("/api/users");
stringImportedGet("/api/users");
ImportedHeaders.get("x");
tsAxios.get("/api/users");
undiciRequest("https://example.com/users");
playwrightRequest.get("/api/users");
post("/api/users");
axios("/api/users");
got("/api/users");
interopAxios.get("/api/users");
require("axios").get("/api/users");
restClient.get("/api/users");
getWithDefault("/api/users");
assignedClient = axios;
assignedClient.get("/api/users");
assignedGet = axios.get;
assignedGet("/api/users");
assignedRequest = axios.request;
assignedRequest("/api/users");
assignedCreate = axios.create;
assignedCreate().get("/api/users");
({ create: assignedCreate } = axios);
assignedCreate().get("/api/users");
({ extend: assignedExtend } = got);
assignedExtend({}).get("/api/users");
({ extend } = got);
extend({}).get("/api/users");
axios?.get("/api/users");
axios?.("/api/users");
axios?.get?.("/api/users");
axios["get"]("/api/users");
axios?.["request"]("/api/users");
undiciRequest?.("https://example.com/users");
axios?.defaults.get("/api/users");
[ignoredAssignmentTarget] = [axios];
optionalAlias.get("/api/users");
reassignedClient = axios;
reassignedClient = { get(_path: string) {} };
reassignedClient.get("/local-reassigned-only");
blockReassignedClient = axios;
{
  blockReassignedClient = { get(_path: string) {} };
}
blockReassignedClient.get("/block-local-reassigned-only");
{
  blockPromotedClient = axios;
}
blockPromotedClient.get("/api/users");
selfAssignedClient = axios;
selfAssignedClient = passthrough(selfAssignedClient.get("/api/users"));
selfAssignedClient.get("/local-self-assigned-only");
computedTarget[selfAssignedClient.get("/api/users")] = (selfAssignedClient = localSource);
{
  const request = axios.create;
  request("/local-cross-category-factory-only");
}
logicalClient = axios;
logicalClient ||= localSource;
logicalClient.get("/api/users");
nullishClient = axios;
nullishClient ??= localSource;
nullishClient.get("/api/users");
({ get: destructuredGet } = axios);
destructuredGet("/api/users");
({ request: assignedRequest } = axios);
assignedRequest("/api/users");
({ get: destructuredGet } = localSource);
destructuredGet("/local-destructured-reassigned-only");
({ ...destructuredClient } = axios);
destructuredClient.get("/api/users");
({ put } = axios);
put("/api/users");
({ shadowedDefaults } = axios);
shadowedDefaults.get("/local-shorthand-only");
[assignedArrayClient, ...assignedArrayRestClient] = axios;
assignedArrayClient.get("/local-array-only");
assignedArrayRestClient.get("/local-array-rest-only");
[defaultedArrayClient = axios] = axios;
defaultedArrayClient.get("/local-array-default-only");
({ get: nestedDefaultGet = axios.get } = axios);
nestedDefaultGet("/api/users");
({ defaults: { get: nestedObjectGet } = axios.defaults } = axios);
nestedObjectGet("/local-nested-object-only");
response = axios.request("/api/users");
response.get("/local-response-only");
[[nestedArrayClient]] = axios;
nestedArrayClient.get("/local-nested-array-only");
[{ get: nestedObjectGet }] = axios;
[holder.item] = axios;
holder.direct = axios;
({ assignedGetShorthand } = axios.get);
assignedGetShorthand("/api/users");
implicitClient = axios;
implicitClient.get("/api/users");

headers.get("x");
cjsDefaultAxios.get("/api/users");
cjsDefaultAxios("/api/users");
namedDefaultAxios.get("/api/users");
factoryGet("/api/users");
void nodeFetchUnused;

const NamedClassExpression = class axios {
  method() {
    axios.get("/class-expression-local-only");
  }
};
axios.get("/api/users");
void NamedClassExpression;

function localClient(axios: { get(path: string): void }) {
  axios.get("/local-only");
}

function restLocal(...axios: Array<{ get(path: string): void }>) {
  axios[0].get("/rest-local-only");
}

function defaultParam(axios = { get(_path: string) {} }) {
  axios.get("/default-local-only");
}

const arrowLocal = (axios: { get(path: string): void }, ...rest: unknown[]) => {
  void rest;
  axios.get("/arrow-local-only");
};

const arrowVar = () => {
  var arrowClient = axios;
  arrowClient.get("/api/users");
};

arrowClient.get("/leaked");

const namedFunction = function axios() {
  axios.get("/named-function-local-only");
};

function hoistedVarShadow() {
  axios.get("/hoisted-var-local-only");
  var axios = { get(_path: string) {} };
}

function nestedHoistedVarShadow(flag: boolean) {
  axios.get("/nested-hoisted-var-local-only");
  if (flag) {
    var axios = { get(_path: string) {} };
  } else {
    var axios = { get(_path: string) {} };
  }
}

function loopAndTryHoistedVarShadow() {
  axios.get("/loop-try-hoisted-var-local-only");
  for (var axios = { get(_path: string) {} };;) {
    break;
  }
  try {
    throw new Error("x");
  } catch {
    var axios = { get(_path: string) {} };
  }
}

function nestedFunctionVarsDoNotShadow() {
  const notVar = 1;
  function nested() {
    var axios = { get(_path: string) {} };
  }
  (() => {
    var axios = { get(_path: string) {} };
  });
  class Nested {}
  axios.get("/api/users");
  void notVar;
  void nested;
  void Nested;
}

function lexicalShadowsArePredeclared() {
  axios.get("/tdz-let-local-only");
  let axios = { get(_path: string) {} };
  {
    axios.get("/tdz-const-local-only");
    const axios = { get(_path: string) {} };
    void axios;
  }
  class LocalClient {}
  LocalClient.get("/tdz-class-local-only");
  void axios;
}

function lexicalClassShadowsArePredeclared() {
  axios.get("/tdz-class-local-only");
  class axios {
    static get(_path: string) {}
  }
  void axios;
}

function switchLexicalShadowsArePredeclared(kind: string) {
  switch (kind) {
    case "call":
      axios.get("/switch-local-only");
      break;
    case "shadow":
      const axios = { get(_path: string) {} };
      void axios;
      break;
  }
}

{
  axios.get("/block-function-local-only");
  function axios() {}
}

try {
  throw new Error("x");
} catch (axios) {
  axios.get("/catch-local-only");
}

{
  class axios {
    static get(_path: string) {}
  }
  axios.get("/class-local-only");
}

function varClient() {
  var local = axios;
  local.get("/api/users");
}

{
  const blockClient = axios;
  blockClient.get("/api/users");
}

void defaults;
void computedTarget;
void headers;
void AxiosHeaders;
void ImportedHeaders;
void extend;
void ignoredArrayClient;
void shadowedDefaults;
void assignedArrayClient;
void assignedArrayRestClient;
void defaultedArrayClient;
void nestedObjectGet;
void nestedArrayClient;
void localOnly;
void localRest;
void firstLocal;
void restLocalItems;
void ignoredDynamicRequire;
void namedFunction;
localClient({ get() {} });
restLocal({ get() {} });
defaultParam();
arrowLocal({ get() {} });
arrowVar();
switchLexicalShadowsArePredeclared("call");

function passthrough<T>(value: T): T {
  return value;
}
