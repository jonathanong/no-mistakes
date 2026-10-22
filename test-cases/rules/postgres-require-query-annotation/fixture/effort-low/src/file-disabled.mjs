// no-mistakes-disable-file postgres-require-query-annotation: explicit file opt-out
import { read } from "@bench/db";
read("SELECT 1"); // EXPECT_FILE_SUPPRESSED
