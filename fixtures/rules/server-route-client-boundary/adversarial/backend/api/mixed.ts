import axios from "axios";
import * as http from "http";
import { request } from "undici";
import "got";

const app = { get(_path: string, _handler: unknown) {} };
app.get("/users", () => {});
app.get(`/template-users`, () => {});

const alias = axios;
(alias).get("/alias");

axios.default.get("/default");
axios.create().get("/direct-created");

const clientFactory = axios.create;
clientFactory().get("/factory");

(function makeClient() {
  return axios;
})().get("/ignored");

const { get } = axios;
void get;

let lateClient;
void lateClient;

http.get("/http");
request.get("/request");

const chained = require("axios").create;
chained.get("/chained");

const cjsCreated = require("axios").create();
cjsCreated.get("/cjs-created");

const undici = require("undici");
undici.fetch("/undici-fetch");

let reassignedClient = axios;
({ x: reassignedDefault = reassignedClient.get("/stale-default") } = (reassignedClient = {
  get(_path: string) {},
}));
