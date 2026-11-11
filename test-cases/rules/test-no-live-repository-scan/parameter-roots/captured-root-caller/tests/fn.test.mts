import { update } from "../src/update.mts";

declare const target: string;
update(target);

// File-wide consumer control: docs/rules/test-no-live-repository-scan.md.
const gate = { update: 1 };
const { update: hidden } = gate;
void hidden;
