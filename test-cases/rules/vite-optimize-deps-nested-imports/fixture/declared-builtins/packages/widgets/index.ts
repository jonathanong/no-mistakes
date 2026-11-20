import { emitter } from "events";
import browser from "process/browser";
import { shape } from "items";
import path from "path";
import { readFile } from "fs/promises";

export const widget = emitter ?? browser ?? shape ?? path ?? readFile;
