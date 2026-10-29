// Runtime-asset reads under a declared root are listed separately; other
// trees, including near-miss names, stay reported.
import { assetEscape } from "../src/asset-escape";
import { assetParent } from "../src/asset-parent";
import { migrationFiles } from "../src/migration-status";
import { nearMiss } from "../src/near-miss";
import { suppressedMigrations } from "../src/migration-suppressed";
import { sourceTree } from "../src/source-tree";

migrationFiles();
nearMiss();
assetParent();
assetEscape("x");
sourceTree();
suppressedMigrations();
