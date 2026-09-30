import { libFn } from "@scope/lib";
import * as util from "@scope/lib/util";
import { utilFn } from "@scope/lib/util";
import { deepFn } from "@scope/lib/barrel";
import { answer } from "@scope/lib/data";
import { missing } from "@scope/missing";
import { useState } from "react";

export function run() {
  libFn();
  utilFn();
  util.utilFn();
  deepFn();
  answer();
  missing();
  useState();
}
