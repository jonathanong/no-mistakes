import * as api from "@acme/rate-limit";

api[`invalidateAll`]();
const { [`invalidateAll`]: clear } = api;
clear();
