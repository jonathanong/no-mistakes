import { importedTag } from "./tagged-template-target.mts";

function localTag(_strings: TemplateStringsArray) {}

function dynamicTag() {
  return localTag;
}

const tags = { importedTag };

export function taggedTemplates() {
  localTag`local`;
  importedTag`imported`;
  dynamicTag()`dynamic`;
  tags[unknownTag]`computed`;
}
