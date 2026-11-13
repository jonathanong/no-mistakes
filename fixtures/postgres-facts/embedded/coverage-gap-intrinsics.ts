import { query } from "@example/db";
import files = require("node:fs");
import type filesType = require("node:path");

enum Color {
  Red,
}

namespace Widgets {
  export const label = "widgets";
}

query(`SELECT 1`);
void files;
void filesType;
void Color;
void Widgets;
