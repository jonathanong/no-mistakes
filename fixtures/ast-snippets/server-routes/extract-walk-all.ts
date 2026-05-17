import express, { "Router" as StringRouter, Router as ExpressRouter } from "express";
import { Hono } from "hono";
import KoaRouter, { Router as KoaNamed } from "@koa/router";
import pathMatch from "koa-path-match";
import { createApp } from "api-server";

const { ignored } = source;
const ROOT = `/root`;
const ARRAY = "/array";

const api = express();
const router = express.Router();
const stringRouter = StringRouter();
const route = pathMatch();
const hono = new Hono({ prefix: "/hono" });
const koa = new KoaRouter({ prefix: "/koa" });
const namedKoa = new KoaNamed();
const apiServer = createApp();
const base = api.basePath(ROOT);
const routed = api.route("/routed");
const paren = (api).route("/paren");

api.get("/direct");
api.del("/del");
api.use("/mounted", router.routes(), router.middleware(), hono);
api.use(router);
api.route("/api-route", stringRouter);
api.prefix("/prefix");
api.get();
router.get([ARRAY, `/template-array`]);
koa.get("named", ["/named", ROOT]);
hono.on(["GET", "del"], ["/on", ROOT]);
route("/matched").get();
base.get("/child");
routed.post("/post");
paren.put("/put");
apiServer.get("/api-server");

export const exported = router;
export { router as publicRouter };
const defaultThing = router;
export default defaultThing;
