import sql from "@example/sql";
import {aliasColumns as fromPackage} from "third-party-columns";
import * as fs from "node:fs";
import * as paths from "path";
import {unrelated} from "./missing";

declare function consume(value: unknown): void;

consume(fromPackage);
consume(fs);
consume(paths);
consume(unrelated);
