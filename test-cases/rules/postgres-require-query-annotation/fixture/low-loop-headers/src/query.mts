import { read } from "@bench/db";

export function onceInInitializer(enabled: boolean) {
  let query = "/* named */ SELECT 1";
  for (read(query); enabled; ) { // EXPECT_ACCEPTED_INITIALIZER
    query = "SELECT 2";
    break;
  }
}

export function onceInOfRhs() {
  let query = "/* named */ SELECT 3";
  for (const item of read(query)) { // EXPECT_ACCEPTED_OF_RHS
    query = "SELECT 4";
    break;
  }
}

export function onceInInRhs() {
  let query = "/* named */ SELECT 5";
  for (const key in read(query)) { // EXPECT_ACCEPTED_IN_RHS
    query = "SELECT 6";
    break;
  }
}

export function initializerRepeatsThroughOuterLoop(enabled: boolean) {
  let query = "/* named */ SELECT 7";
  while (enabled) {
    for (read(query); enabled; ) { // EXPECT_MISSING_INITIALIZER_REPEATED
      query = "SELECT 8";
      break;
    }
  }
}

export function ofRhsRepeatsThroughOuterLoop(enabled: boolean) {
  let query = "/* named */ SELECT 9";
  while (enabled) {
    for (const item of read(query)) { // EXPECT_MISSING_OF_RHS_REPEATED
      query = "SELECT 10";
      break;
    }
  }
}

export function inRhsRepeatsThroughOuterLoop(enabled: boolean) {
  let query = "/* named */ SELECT 11";
  while (enabled) {
    for (const key in read(query)) { // EXPECT_MISSING_IN_RHS_REPEATED
      query = "SELECT 12";
      break;
    }
  }
}

export function updateCallsAfterBodyWrite(enabled: boolean) {
  let query = "/* named */ SELECT 13";
  for (
    read(query); // EXPECT_ACCEPTED_UPDATE_INITIALIZER
    enabled;
    read(query) // EXPECT_MISSING_UPDATE_CALL
  ) {
    query = "SELECT 14";
  }
}
