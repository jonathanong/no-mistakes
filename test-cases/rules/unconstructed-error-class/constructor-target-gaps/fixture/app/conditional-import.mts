import {Choice, Opposite} from "conditional-entry";
export class UnusedImportError extends Choice {}
export class ImportPlainChild extends Opposite {}
new Choice();
new Opposite();
