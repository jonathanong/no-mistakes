import { test as base } from "@playwright/test";
import { test as broken } from "./broken";

export let test = Math.random() > 0.5 ? base : broken;
