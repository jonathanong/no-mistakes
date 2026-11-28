declare const moduleName: string;

var api = require("./data");
const { getWidget: read } = api;
export async function throughVar(): Promise<Widget | null> {
  return read();
}

var rebound = require("./data");
rebound = require("./other");
const { getWidget: moved } = rebound;
export async function reassigned(): Promise<Widget | null> {
  return moved();
}

export async function inline(): Promise<Widget | null> {
  return require(moduleName)("1");
}

export async function staticInline(): Promise<Widget | null> {
  return require("./data")("1");
}

declare const list: { head: { next: { focus(): Promise<Widget | null> } } };

export async function selfMemberWalk(): Promise<Widget | null> {
  let node = list.head;
  node = node.next;
  return node.focus();
}

export async function mutualMemberWalk(): Promise<Widget | null> {
  var linked = mutual?.x;
  var mutual = linked?.y;
  return linked.focus();
}
