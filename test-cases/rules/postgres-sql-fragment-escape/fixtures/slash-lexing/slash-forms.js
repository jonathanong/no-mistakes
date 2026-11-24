import { widgetColumns } from "./columns";

const record = (value) => value;
const size = 12;
const sizes = [size];
let count = size;
const afterParen = () => 2;
const afterBracket = () => 2;
const afterBrace = () => 2;
const afterIncrement = () => 2;

record(/[)'"] require("marker")/g);
record((size) / afterParen() / 2);
record(sizes[0] / afterBracket() / 2);
record({ valueOf: () => 12 } / afterBrace() / 2);
record(count++ / afterIncrement() / 2);

export function extend(extra) {
  widgetColumns.append(extra);
}

const singleDivisor = () => 2;
record(count++ / singleDivisor());
