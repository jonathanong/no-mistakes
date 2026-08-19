import Fastify from "fastify";
import { fastify as namedFastify } from "fastify";

const app = Fastify();
app.get("/health", async () => ({ ok: true }));
app.post("/users", async () => ({ id: 1 }));

const named = namedFastify();
named.delete("/named", async () => ({ ok: true }));

const cjs = require("fastify")();
cjs.put("/cjs", async () => ({ ok: true }));
