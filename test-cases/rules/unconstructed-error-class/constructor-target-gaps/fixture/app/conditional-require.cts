import {Choice, Opposite} from "conditional-entry";
export class RequirePlainChild extends Choice {}
export class UnusedRequireError extends Opposite {}
new Choice();
new Opposite();
