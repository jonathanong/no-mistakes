import fetchItem from "@example/http";
import { default as namedDefault } from "@example/http";
import { send, request } from "@example/transport";
import * as agents from "@example/agents";
import { otherAgent } from "@example/other";
import agentsEq = require("@example/agents");

const { otherAgent: cjsAgent } = require("@example/agents");

fetchItem();
namedDefault();
send({ agent: cjsAgent() });
send({ agent: agentsEq.otherAgent() });
(agents as typeof agents).createUnsafeAgent({ agent: otherAgent() });
request["call" as const](null);
const { request: req } = (require as NodeRequire)("@example/transport");
req();
