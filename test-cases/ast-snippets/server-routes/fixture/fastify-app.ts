import Fastify from "fastify";
import { fastify as namedFastify, Fastify as FastifyCtor } from "fastify";
import FastifyReq = require("fastify");

const app = Fastify();
app.get("/health", async () => ({ ok: true }));
app.post("/users", async () => ({ id: 1 }));

const named = namedFastify();
named.delete("/named", async () => ({ ok: true }));

const ctor = FastifyCtor();
ctor.head("/ctor", async () => ({ ok: true }));

const equals = FastifyReq();
equals.patch("/equals", async () => ({ ok: true }));

const cjs = require("fastify")();
cjs.put("/cjs", async () => ({ ok: true }));
