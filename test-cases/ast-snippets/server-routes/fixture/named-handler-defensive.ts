import express from "express";

const app = express();

declare function declaredHandler(req, res): void;
export declare function exportedDeclaredHandler(req, res): void;

export default function (req, res) {
  req.query.defaultAnonymous;
  res.send("ok");
}

app.get("/declared", declaredHandler);
