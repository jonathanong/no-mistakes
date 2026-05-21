import express from "express";

const app = express();
const UNUSED_ROUTE = "/unused";
const USERS_ROUTE = `/users`;

void UNUSED_ROUTE;
export const registered = app.get(USERS_ROUTE, () => {});
