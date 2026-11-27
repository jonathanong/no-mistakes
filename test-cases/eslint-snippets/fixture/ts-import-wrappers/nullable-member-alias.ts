declare const key: string;
const read = require("./data").getWidget;
const missing = require("./data")[key];

export async function named(): Promise<Widget | null> {
  return read("1");
}

export async function hidden(): Promise<Widget | null> {
  return missing("1");
}
