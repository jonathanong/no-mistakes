import { readdirSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
const root = mkdtempSync(join(tmpdir(), "bounded-"));
const project = {next:{next:{next:{next:{next:{next:{next:{next:{next:{next:{next:{next:{next:{next:{next:{next:{next:{next:{root:"src"}}}}}}}}}}}}}}}}}}};
readdirSync(project.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.next.root ?? root);
