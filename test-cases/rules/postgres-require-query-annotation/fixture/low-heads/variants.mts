import { read } from "@bench/db";
// Variant count is irrelevant to original low-effort head syntax.
const choices16 = flag0
  ? "/* branch 0 */ SELECT 0"
  : flag1
    ? "/* branch 1 */ SELECT 1"
    : flag2
      ? "/* branch 2 */ SELECT 2"
      : flag3
        ? "/* branch 3 */ SELECT 3"
        : flag4
          ? "/* branch 4 */ SELECT 4"
          : flag5
            ? "/* branch 5 */ SELECT 5"
            : flag6
              ? "/* branch 6 */ SELECT 6"
              : flag7
                ? "/* branch 7 */ SELECT 7"
                : flag8
                  ? "/* branch 8 */ SELECT 8"
                  : flag9
                    ? "/* branch 9 */ SELECT 9"
                    : flag10
                      ? "/* branch 10 */ SELECT 10"
                      : flag11
                        ? "/* branch 11 */ SELECT 11"
                        : flag12
                          ? "/* branch 12 */ SELECT 12"
                          : flag13
                            ? "/* branch 13 */ SELECT 13"
                            : flag14
                              ? "/* branch 14 */ SELECT 14"
                              : "/* branch 15 */ SELECT 15";
read(choices16); // low: false
read("/* literal head */ " + choices16); // low: true
const choices17 = flag0
  ? "/* branch 0 */ SELECT 0"
  : flag1
    ? "/* branch 1 */ SELECT 1"
    : flag2
      ? "/* branch 2 */ SELECT 2"
      : flag3
        ? "/* branch 3 */ SELECT 3"
        : flag4
          ? "/* branch 4 */ SELECT 4"
          : flag5
            ? "/* branch 5 */ SELECT 5"
            : flag6
              ? "/* branch 6 */ SELECT 6"
              : flag7
                ? "/* branch 7 */ SELECT 7"
                : flag8
                  ? "/* branch 8 */ SELECT 8"
                  : flag9
                    ? "/* branch 9 */ SELECT 9"
                    : flag10
                      ? "/* branch 10 */ SELECT 10"
                      : flag11
                        ? "/* branch 11 */ SELECT 11"
                        : flag12
                          ? "/* branch 12 */ SELECT 12"
                          : flag13
                            ? "/* branch 13 */ SELECT 13"
                            : flag14
                              ? "/* branch 14 */ SELECT 14"
                              : flag15
                                ? "/* branch 15 */ SELECT 15"
                                : "/* branch 16 */ SELECT 16";
read(choices17); // low: false
read("/* literal head */ " + choices17); // low: true
