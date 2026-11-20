import path from "path";
import { readFileSync } from "fs";
import { readFile } from "fs/promises";
import nodeFs from "node:fs";
import widgets from "widgets";
import { shape } from "items";
import { label } from "orders";
import { tag } from "labels";

const crypto = require("crypto");

export const widget = {
  path,
  readFileSync,
  readFile,
  nodeFs,
  widgets,
  shape,
  label,
  tag,
  crypto,
};
