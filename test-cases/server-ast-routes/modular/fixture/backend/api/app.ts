import express from "express";
import admin from "./admin";
import { members } from "./users";
import { legacy } from "./legacy";
import equalsMembers = require("./users");

const app = express();

app.use("/api", requireAuth, members);
app.use("/equals", equalsMembers);
app.use("/legacy", legacy);
app.use(admin);

function requireAuth() {}
