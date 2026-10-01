// A file with no import or export is a script, and its namespace is global:
// any other file can reach it without an import. Nothing here is an export of
// the file, so nothing is reported.
namespace ScriptNamespace {
  // Not flagged: not an export of the file.
  export class ScriptDead extends Error {}
}
