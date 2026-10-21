import { read } from '@bench/db'
// Shared snapshots retain these labels across speculative branches.
const record = {
  long_repeated_record_label_000: "value",
  long_repeated_record_label_001: "value",
  long_repeated_record_label_002: "value",
  long_repeated_record_label_003: "value",
  long_repeated_record_label_004: "value",
  long_repeated_record_label_005: "value",
  long_repeated_record_label_006: "value",
  long_repeated_record_label_007: "value",
  long_repeated_record_label_008: "value",
  long_repeated_record_label_009: "value",
  long_repeated_record_label_010: "value",
  long_repeated_record_label_011: "value",
  long_repeated_record_label_012: "value",
  long_repeated_record_label_013: "value",
  long_repeated_record_label_014: "value",
  long_repeated_record_label_015: "value",
  long_repeated_record_label_016: "value",
  long_repeated_record_label_017: "value",
  long_repeated_record_label_018: "value",
  long_repeated_record_label_019: "value",
  long_repeated_record_label_020: "value",
  long_repeated_record_label_021: "value",
  long_repeated_record_label_022: "value",
  long_repeated_record_label_023: "value",
  long_repeated_record_label_024: "value",
  long_repeated_record_label_025: "value",
  long_repeated_record_label_026: "value",
  long_repeated_record_label_027: "value",
  long_repeated_record_label_028: "value",
  long_repeated_record_label_029: "value",
  long_repeated_record_label_030: "value",
  long_repeated_record_label_031: "value",
  long_repeated_record_label_032: "value",
  long_repeated_record_label_033: "value",
  long_repeated_record_label_034: "value",
  long_repeated_record_label_035: "value",
  long_repeated_record_label_036: "value",
  long_repeated_record_label_037: "value",
  long_repeated_record_label_038: "value",
  long_repeated_record_label_039: "value",
  long_repeated_record_label_040: "value",
  long_repeated_record_label_041: "value",
  long_repeated_record_label_042: "value",
  long_repeated_record_label_043: "value",
  long_repeated_record_label_044: "value",
  long_repeated_record_label_045: "value",
  long_repeated_record_label_046: "value",
  long_repeated_record_label_047: "value",
}
const shared0 = record
const shared1 = record
const shared2 = record
const shared3 = record
const shared4 = record
const shared5 = record
const shared6 = record
const shared7 = record
const shared8 = record
const shared9 = record
const shared10 = record
const shared11 = record
const shared12 = record
const shared13 = record
const shared14 = record
const shared15 = record
const shared16 = record
const shared17 = record
const shared18 = record
const shared19 = record
const shared20 = record
const shared21 = record
const shared22 = record
const shared23 = record
// These expressions force arm joins during module initialization.
flag ? shared0 : shared0
flag ? shared1 : shared1
flag ? shared2 : shared2
flag ? shared3 : shared3
flag ? shared4 : shared4
flag ? shared5 : shared5
flag ? shared6 : shared6
flag ? shared7 : shared7
flag ? shared8 : shared8
flag ? shared9 : shared9
flag ? shared10 : shared10
flag ? shared11 : shared11
flag ? shared12 : shared12
flag ? shared13 : shared13
flag ? shared14 : shared14
flag ? shared15 : shared15
flag ? shared16 : shared16
flag ? shared17 : shared17
flag ? shared18 : shared18
flag ? shared19 : shared19
flag ? shared20 : shared20
flag ? shared21 : shared21
flag ? shared22 : shared22
flag ? shared23 : shared23
export function probe() {
  read("/* clean */ SELECT 1") // safe-query
  read("SELECT 2") // unsafe-query
  return record
}
