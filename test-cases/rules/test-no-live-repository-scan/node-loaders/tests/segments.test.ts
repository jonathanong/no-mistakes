import { readdirSync } from "node:fs";
readdirSync(".github/fixtures/../workflows"); // escaped-segment
readdirSync(".github/fixtures/x"); // bounded-segment
