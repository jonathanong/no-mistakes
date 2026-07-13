import express from "express";
import { router } from "@server/router";

const app = express();
app.use("/api", router);
