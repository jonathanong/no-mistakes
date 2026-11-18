import type { SessionExecutor } from "@example/db";
import {
  parameterTarget,
  propertyRead,
  captureInput,
  getterSource,
  spreadSource,
  effectfulStatus,
  computedKey,
  methodSource,
  aliasedMetadata,
  optionalAssign,
  elementAssign,
} from "./fresh-error-control-helpers";
import { shadowedObject } from "./fresh-error-shadow-helper";
import { byId, type Input } from "./query";

export async function afterParameterTarget(query: SessionExecutor, notice: Input | undefined) {
  parameterTarget(notice, 404);
  return byId(query, notice.id);
}

export async function afterPropertyRead(query: SessionExecutor, notice: Input | undefined) {
  propertyRead(notice, 404);
  return byId(query, notice.id);
}

export async function afterCapture(query: SessionExecutor, notice: Input | undefined) {
  captureInput(notice, 404);
  return byId(query, notice.id);
}

export async function afterGetter(query: SessionExecutor, notice: Input | undefined) {
  getterSource(notice, 404);
  return byId(query, notice.id);
}

export async function afterSpread(query: SessionExecutor, notice: Input | undefined) {
  spreadSource(notice, 404);
  return byId(query, notice.id);
}

export async function afterEffectfulStatus(query: SessionExecutor, notice: Input | undefined) {
  effectfulStatus(notice, 404);
  return byId(query, notice.id);
}

export async function afterShadow(query: SessionExecutor, notice: Input | undefined) {
  shadowedObject(notice, 404);
  return byId(query, notice.id);
}

export async function afterComputedKey(query: SessionExecutor, notice: Input | undefined) {
  computedKey(notice, 404);
  return byId(query, notice.id);
}

export async function afterMethod(query: SessionExecutor, notice: Input | undefined) {
  methodSource(notice, 404);
  return byId(query, notice.id);
}

export async function afterMetadataAlias(query: SessionExecutor, notice: Input | undefined) {
  aliasedMetadata(notice, 404);
  return byId(query, notice.id);
}

export async function afterOptionalAssign(query: SessionExecutor, notice: Input | undefined) {
  optionalAssign(notice, 404);
  return byId(query, notice.id);
}

export async function afterElementAssign(query: SessionExecutor, notice: Input | undefined) {
  elementAssign(notice, 404);
  return byId(query, notice.id);
}
