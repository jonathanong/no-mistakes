import { featureValue } from "@scope/pkg/feature";
import { packageValue } from "@scope/pkg";
import { productionValue } from "./api.mjs";

export const used = packageValue() + featureValue() + productionValue();
