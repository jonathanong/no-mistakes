import {aliasColumns} from "@example/aliases/columns";
import {other} from "./other-missing";
import {columns} from "./missing-columns";
import {aliasColumns as fromPackage} from "third-party-columns";
import * as fs from "node:fs";
import * as paths from "path";

declare function consume(value: unknown): void;

consume(columns);
consume(other);
consume(fromPackage);
consume(fs);
consume(paths);
