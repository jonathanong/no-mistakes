import { readdirSync } from "node:fs";

const missingPaths: string[] = [];
readdirSync(...missingPaths);
readdirSync(...[]);
readdirSync();

declare function getRoot(): string;
const unknownRoot: string = getRoot();
readdirSync(unknownRoot);

const livePaths = ["."];
readdirSync(...livePaths);

const fixturePaths = ["./fixtures"];
readdirSync(...fixturePaths);
readdirSync("./fixtures");
