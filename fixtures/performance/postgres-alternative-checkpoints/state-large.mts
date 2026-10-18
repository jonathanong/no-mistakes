// Forty retained argument slots and thirty lexical values; repeated writes hit one slot.
import { query } from "@bench/db";
declare const choose: boolean;
const lexical00 = 0;
const lexical01 = 1;
const lexical02 = 2;
const lexical03 = 3;
const lexical04 = 4;
const lexical05 = 5;
const lexical06 = 6;
const lexical07 = 7;
const lexical08 = 8;
const lexical09 = 9;
const lexical10 = 10;
const lexical11 = 11;
const lexical12 = 12;
const lexical13 = 13;
const lexical14 = 14;
const lexical15 = 15;
const lexical16 = 16;
const lexical17 = 17;
const lexical18 = 18;
const lexical19 = 19;
const lexical20 = 20;
const lexical21 = 21;
const lexical22 = 22;
const lexical23 = 23;
const lexical24 = 24;
const lexical25 = 25;
const lexical26 = 26;
const lexical27 = 27;
const lexical28 = 28;
const lexical29 = 29;

function payload(
  untouched00: string, target: string, untouched01: string, untouched02: string,
  untouched03: string, untouched04: string, untouched05: string, untouched06: string,
  untouched07: string, untouched08: string, untouched09: string, untouched10: string,
  untouched11: string, untouched12: string, untouched13: string, untouched14: string,
  untouched15: string, untouched16: string, untouched17: string, untouched18: string,
  untouched19: string, untouched20: string, untouched21: string, untouched22: string,
  untouched23: string, untouched24: string, untouched25: string, untouched26: string,
  untouched27: string, untouched28: string, untouched29: string, untouched30: string,
  untouched31: string, untouched32: string, untouched33: string, untouched34: string,
  untouched35: string, untouched36: string, untouched37: string, untouched38: string
) {
  const reader = () => arguments[1];
  choose
    ? ((arguments[1] = "intermediate left"), (arguments[1] = "SELECT 1 /* left */"))
    : ((arguments[1] = "intermediate right"), (arguments[1] = "SELECT 2 /* right */"));
  return reader;
}

const reader = payload(
  "value-00", "initial", "value-01", "value-02",
  "value-03", "value-04", "value-05", "value-06",
  "value-07", "value-08", "value-09", "value-10",
  "value-11", "value-12", "value-13", "value-14",
  "value-15", "value-16", "value-17", "value-18",
  "value-19", "value-20", "value-21", "value-22",
  "value-23", "value-24", "value-25", "value-26",
  "value-27", "value-28", "value-29", "value-30",
  "value-31", "value-32", "value-33", "value-34",
  "value-35", "value-36", "value-37", "value-38"
);
export const selected = query(reader());
