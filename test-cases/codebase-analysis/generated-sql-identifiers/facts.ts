import cryptoDefault, { randomBytes, randomBytes as bytes } from "node:crypto";
import * as cryptoNamespace from "crypto";
import foreignCrypto, { randomBytes as foreignBytes, other as wrongNamed } from "other";
import { other as wrongCryptoNamed } from "node:crypto";
import type { randomBytes as typeBytes } from "node:crypto";
import { type randomBytes as inlineTypeBytes } from "node:crypto";
import { randomBytes as duplicate } from "node:crypto";
import { randomBytes as duplicate } from "other";

const directName = `tmp_${randomBytes(8).toString("hex")}`;
const aliasName = "tmp_" + bytes(8).toString("hex");
const defaultName = `tmp_${cryptoDefault.randomBytes(8).toString("hex")}`;
const namespaceName = `tmp_${cryptoNamespace.randomBytes(8).toString("hex")}`;
const suffixName = `tmp_${bytes(8).toString("hex")}_$42`;
const zeroName = `tmp_${bytes(0).toString("hex")}`;
const wrappedName = (`tmp_` as const) + (bytes((0x8 as number)).toString(`hex`) as string);

export function scoped() {
  const scopedName = `tmp_${bytes(8).toString("hex")}`;
  return scopedName;
}

const noPrefix = `${bytes(8).toString("hex")}`;
const emptyPrefix = "" + bytes(8).toString("hex");
const invalidPrefix = `tmp-${bytes(8).toString("hex")}`;
const numericPrefix = `2tmp_${bytes(8).toString("hex")}`;
const invalidSuffix = `tmp_${bytes(8).toString("hex")};`;
const openSuffix = `tmp_${bytes(8).toString("hex")}_${process.env.SUFFIX}`;
const notAddition = "tmp_" - bytes(8).toString("hex");
const notTemplate = bytes(8).toString("hex");
const notBytes = `tmp_${cryptoDefault.digest(8).toString("hex")}`;
const nestedReceiver = `tmp_${cryptoDefault.sub.randomBytes(8).toString("hex")}`;
const computedBytes = `tmp_${cryptoDefault["randomBytes"](8).toString("hex")}`;
const computedConversion = `tmp_${bytes(8)["toString"]("hex")}`;
const wrongConversion = `tmp_${bytes(8).join("hex")}`;
const wrongEncoding = `tmp_${bytes(8).toString("base64")}`;
const missingEncoding = `tmp_${bytes(8).toString()}`;
const extraEncoding = `tmp_${bytes(8).toString("hex", 0)}`;
const dynamicEncoding = `tmp_${bytes(8).toString(process.env.ENCODING)}`;
const spreadEncoding = `tmp_${bytes(8).toString(...["hex"])}`;
const noSize = `tmp_${bytes().toString("hex")}`;
const dynamicSize = `tmp_${bytes(size).toString("hex")}`;
const stringSize = `tmp_${bytes("8").toString("hex")}`;
const fractionalSize = `tmp_${bytes(1.5).toString("hex")}`;
const negativeSize = `tmp_${bytes(-8).toString("hex")}`;
const infiniteSize = `tmp_${bytes(1e999).toString("hex")}`;
const callback = `tmp_${bytes(8, () => {}).toString("hex")}`;
const spreadSize = `tmp_${bytes(...[8]).toString("hex")}`;
const optionalBytes = `tmp_${bytes?.(8).toString("hex")}`;
const optionalConversion = `tmp_${bytes(8).toString?.("hex")}`;
const optionalReceiver = `tmp_${cryptoDefault?.randomBytes(8).toString("hex")}`;
const foreignDefault = `tmp_${foreignCrypto.randomBytes(8).toString("hex")}`;
const foreignNamed = `tmp_${foreignBytes(8).toString("hex")}`;
const defaultCalled = `tmp_${cryptoDefault(8).toString("hex")}`;
const namedReceiver = `tmp_${bytes.randomBytes(8).toString("hex")}`;
const wrongImport = `tmp_${wrongCryptoNamed(8).toString("hex")}`;
const wrongMember = `tmp_${wrongNamed.randomBytes(8).toString("hex")}`;
const typeOnly = `tmp_${typeBytes(8).toString("hex")}`;
const inlineTypeOnly = `tmp_${inlineTypeBytes(8).toString("hex")}`;
const duplicateImports = `tmp_${duplicate(8).toString("hex")}`;
const globalCrypto = `tmp_${crypto.randomBytes(8).toString("hex")}`;
const unimported = `tmp_${unimportedBytes(8).toString("hex")}`;

export function shadowed(bytes: (size: number) => Buffer, cryptoDefault: typeof foreignCrypto) {
  const parameterName = `tmp_${bytes(8).toString("hex")}`;
  const parameterReceiver = `tmp_${cryptoDefault.randomBytes(8).toString("hex")}`;
  return [parameterName, parameterReceiver];
}

export function blockShadow() {
  {
    const blockName = `tmp_${bytes(8).toString("hex")}`;
    const bytes = foreignBytes;
    return blockName;
  }
}

export function namespaceShadow() {
  const cryptoNamespace = foreignCrypto;
  const namespaceShadowName = `tmp_${cryptoNamespace.randomBytes(8).toString("hex")}`;
  return namespaceShadowName;
}
