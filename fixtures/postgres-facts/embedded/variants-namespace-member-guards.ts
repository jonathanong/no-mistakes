import { query } from "@example/db";
import sql from "sql-template-strings";

type Mode = "a" | "b" | "all";

namespace Reports {
  export type Mode = string;
}
namespace Reports {
  export function mergedOpenMode(options: { mode: Mode }) {
    const statement = sql`SELECT id FROM items`;
    if (options.mode !== "b") {
      statement.append(sql` WHERE a = 1`);
    }
    if (options.mode !== "a") {
      statement.append(sql` ORDER BY id`);
    }
    query(statement);
  }
}
namespace Unshadowed {
  export type Other = string;
}
namespace Unshadowed {
  export function outerMode(options: { mode: Mode }) {
    const statement = sql`SELECT id FROM items`;
    if (options.mode !== "b") {
      statement.append(sql` WHERE a = 1`);
    }
    if (options.mode !== "a") {
      statement.append(sql` ORDER BY id`);
    }
    query(statement);
  }
}
namespace PrivateType {
  type Mode = string;
}
namespace PrivateType {
  export function outerMode(options: { mode: Mode }) {
    const statement = sql`SELECT id FROM items`;
    if (options.mode !== "b") {
      statement.append(sql` WHERE a = 1`);
    }
    if (options.mode !== "a") {
      statement.append(sql` ORDER BY id`);
    }
    query(statement);
  }
}
namespace OwnType {
  export type Other = string;
}
namespace OwnType {
  export type Mode = "a" | "b" | "all";
  export function ownMode(options: { mode: Mode }) {
    const statement = sql`SELECT id FROM items`;
    if (options.mode !== "b") {
      statement.append(sql` WHERE a = 1`);
    }
    if (options.mode !== "a") {
      statement.append(sql` ORDER BY id`);
    }
    query(statement);
  }
}
namespace Nested {
  export namespace Inner {
    export type Mode = string;
  }
}
namespace Nested {
  export namespace Inner {
    export function nestedMergedMode(options: { mode: Mode }) {
      const statement = sql`SELECT id FROM items`;
      if (options.mode !== "b") {
        statement.append(sql` WHERE a = 1`);
      }
      if (options.mode !== "a") {
        statement.append(sql` ORDER BY id`);
      }
      query(statement);
    }
  }
}
namespace Dotted.Inner {
  export type Mode = string;
}
namespace Dotted.Inner {
  export function dottedMergedMode(options: { mode: Mode }) {
    const statement = sql`SELECT id FROM items`;
    if (options.mode !== "b") {
      statement.append(sql` WHERE a = 1`);
    }
    if (options.mode !== "a") {
      statement.append(sql` ORDER BY id`);
    }
    query(statement);
  }
}
namespace ParentMode {
  export type Mode = string;
}
namespace ParentMode.Inner {
  export function parentMergedMode(options: { mode: Mode }) {
    const statement = sql`SELECT id FROM items`;
    if (options.mode !== "b") {
      statement.append(sql` WHERE a = 1`);
    }
    if (options.mode !== "a") {
      statement.append(sql` ORDER BY id`);
    }
    query(statement);
  }
}

namespace InterfaceType {
  export interface Mode { kind: string }
}
namespace InterfaceType {
  export function mergedMode(options: { mode: Mode }) {
    const statement = sql`SELECT id FROM items`;
    if (options.mode !== ("b" as unknown)) {
      statement.append(sql` WHERE a = 1`);
    }
    if (options.mode !== ("a" as unknown)) {
      statement.append(sql` ORDER BY id`);
    }
    query(statement);
  }
}

namespace EnumType {
  export enum Mode { Unknown }
}
namespace EnumType {
  export function mergedMode(options: { mode: Mode }) {
    const statement = sql`SELECT id FROM items`;
    if (options.mode !== ("b" as unknown)) {
      statement.append(sql` WHERE a = 1`);
    }
    if (options.mode !== ("a" as unknown)) {
      statement.append(sql` ORDER BY id`);
    }
    query(statement);
  }
}

namespace ClassType {
  export class Mode {}
}
namespace ClassType {
  export function mergedMode(options: { mode: Mode }) {
    const statement = sql`SELECT id FROM items`;
    if (options.mode !== ("b" as unknown)) {
      statement.append(sql` WHERE a = 1`);
    }
    if (options.mode !== ("a" as unknown)) {
      statement.append(sql` ORDER BY id`);
    }
    query(statement);
  }
}

namespace ImportTarget {
  export class Mode {}
}
namespace ImportedType {
  export import Mode = ImportTarget.Mode;
}
namespace ImportedType {
  export function mergedMode(options: { mode: Mode }) {
    const statement = sql`SELECT id FROM items`;
    if (options.mode !== ("b" as unknown)) {
      statement.append(sql` WHERE a = 1`);
    }
    if (options.mode !== ("a" as unknown)) {
      statement.append(sql` ORDER BY id`);
    }
    query(statement);
  }
}

declare namespace AmbientType {
  type Mode = string;
}
namespace AmbientType {
  export function mergedMode(options: { mode: Mode }) {
    const statement = sql`SELECT id FROM items`;
    if (options.mode !== ("b" as unknown)) {
      statement.append(sql` WHERE a = 1`);
    }
    if (options.mode !== ("a" as unknown)) {
      statement.append(sql` ORDER BY id`);
    }
    query(statement);
  }
}

export namespace ReverseType {
  export function mergedMode(options: { mode: Mode }) {
    const statement = sql`SELECT id FROM items`;
    if (options.mode !== ("b" as unknown)) {
      statement.append(sql` WHERE a = 1`);
    }
    if (options.mode !== ("a" as unknown)) {
      statement.append(sql` ORDER BY id`);
    }
    query(statement);
  }
}
export namespace ReverseType {
  export type Mode = string;
}

namespace PrivateNested {
  export type Mode = string;
}
namespace PrivateNested {
  namespace Local {
    export function mergedMode(options: { mode: Mode }) {
      const statement = sql`SELECT id FROM items`;
      if (options.mode !== ("b" as unknown)) {
        statement.append(sql` WHERE a = 1`);
      }
      if (options.mode !== ("a" as unknown)) {
        statement.append(sql` ORDER BY id`);
      }
      query(statement);
    }
  }
}

namespace PrivateNestedFinite {
  namespace Local {
    export type Mode = string;
  }
}
namespace PrivateNestedFinite {
  namespace Local {
    export function outerMode(options: { mode: Mode }) {
      const statement = sql`SELECT id FROM items`;
      if (options.mode !== ("b" as unknown)) {
        statement.append(sql` WHERE a = 1`);
      }
      if (options.mode !== ("a" as unknown)) {
        statement.append(sql` ORDER BY id`);
      }
      query(statement);
    }
  }
}

declare global {
  interface ExternalMarker {}
  namespace SharedReports {
    type Mode = string;
  }
}
export function afterUnsupportedOwners(options: { mode: Mode }) {
  const statement = sql`SELECT id FROM items`;
  if (options.mode !== "b") {
    statement.append(sql` WHERE a = 1`);
  }
  if (options.mode !== "a") {
    statement.append(sql` ORDER BY id`);
  }
  query(statement);
}

export namespace Augmentable {
  export function outerFallbackIsUnknown(options: { mode: Mode }) {
    const statement = sql`SELECT id FROM items`;
    if (options.mode !== "b") {
      statement.append(sql` WHERE a = 1`);
    }
    if (options.mode !== "a") {
      statement.append(sql` ORDER BY id`);
    }
    query(statement);
  }
}

export namespace ExportedOwn {
  export type Mode = "a" | "b" | "all";
  export function ownFiniteAlias(options: { mode: Mode }) {
    const statement = sql`SELECT id FROM items`;
    if (options.mode !== "b") {
      statement.append(sql` WHERE a = 1`);
    }
    if (options.mode !== "a") {
      statement.append(sql` ORDER BY id`);
    }
    query(statement);
  }
}

namespace NamedExport {
  export function outerFallbackIsUnknown(options: { mode: Mode }) {
    const statement = sql`SELECT id FROM items`;
    if (options.mode !== "b") {
      statement.append(sql` WHERE a = 1`);
    }
    if (options.mode !== "a") {
      statement.append(sql` ORDER BY id`);
    }
    query(statement);
  }
}
export { NamedExport };

namespace PrivateParent {
  export namespace PublicChild {
    export function outerAliasStaysFinite(options: { mode: Mode }) {
      const statement = sql`SELECT id FROM items`;
      if (options.mode !== "b") {
        statement.append(sql` WHERE a = 1`);
      }
      if (options.mode !== "a") {
        statement.append(sql` ORDER BY id`);
      }
      query(statement);
    }
  }
}

export namespace PublicParent {
  export namespace PublicChild {
    export function outerFallbackIsUnknown(options: { mode: Mode }) {
      const statement = sql`SELECT id FROM items`;
      if (options.mode !== "b") {
        statement.append(sql` WHERE a = 1`);
      }
      if (options.mode !== "a") {
        statement.append(sql` ORDER BY id`);
      }
      query(statement);
    }
  }
}
