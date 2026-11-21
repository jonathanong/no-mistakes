import { shape } from "items";
import { tag } from "labels";
import { label } from "orders";
import "items/theme";
import type { Label } from "labels";
export type { Label } from "labels";

export const widget = shape ?? label ?? tag;
