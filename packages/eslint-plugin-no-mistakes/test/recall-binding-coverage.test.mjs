import { describe, expect, it } from "vitest";
import { require } from "./helpers.mjs";

const { fetchInvocation, isFetchCall } = require("../src/helpers.js");
const { expressionName } = require("../src/rules/module-mock-framework.js");
const { logicalCall, memberPropertyName } = require("../src/rules/module-mock-helpers.js");

function context(variables, upper = null) {
  return {
    sourceCode: {
      getScope() {
        return { variables, upper };
      },
    },
  };
}

function ident(name) {
  return { type: "Identifier", name };
}

function variable(name, def) {
  return { name, defs: [def] };
}

const fetchIdent = ident("fetch");
const globalContext = context([]);

describe("fetch invocation recall branches", () => {
  it("rejects calls that are not fetch invocations", () => {
    expect(fetchInvocation(null, globalContext)).toBeNull();
    expect(fetchInvocation({ type: "CallExpression" }, globalContext)).toBeNull();
    expect(
      fetchInvocation(
        { type: "ArrayExpression", callee: fetchIdent, arguments: [] },
        globalContext,
      ),
    ).toBeNull();
    expect(isFetchCall({ type: "CallExpression", callee: ident("axios") }, {})).toBe(false);
    expect(
      fetchInvocation(
        {
          type: "CallExpression",
          callee: { type: "CallExpression", callee: ident("get"), arguments: [] },
          arguments: [],
        },
        globalContext,
      ),
    ).toBeNull();
  });

  it("reads static fetch members, aliases, and forwarded arguments", () => {
    expect(
      fetchInvocation({ type: "CallExpression", callee: fetchIdent, arguments: [] }, globalContext),
    ).toEqual([]);

    const template = {
      type: "TemplateLiteral",
      expressions: [],
      quasis: [{ value: { cooked: "fetch", raw: "fetch" } }],
    };
    const member = (property, object) => ({
      type: "CallExpression",
      callee: { type: "MemberExpression", property, object },
      arguments: [{ type: "Literal", value: "/api" }],
    });
    expect(
      fetchInvocation(
        member({ type: "Literal", value: "fetch" }, ident("globalThis")),
        globalContext,
      ),
    ).toHaveLength(1);
    expect(
      fetchInvocation(member({ type: "Literal", value: 1 }, ident("globalThis")), globalContext),
    ).toBeNull();
    expect(fetchInvocation(member(template, ident("self")), globalContext)).toHaveLength(1);
    expect(fetchInvocation(member(null, ident("global")), globalContext)).toBeNull();
    expect(
      fetchInvocation(
        member(ident("fetch"), {
          type: "ChainExpression",
          expression: ident("window"),
        }),
        globalContext,
      ),
    ).toHaveLength(1);

    const shadowed = context([variable("window", { type: "Variable", node: {} })]);
    expect(
      fetchInvocation(member({ type: "Literal", value: "fetch" }, ident("window")), shadowed),
    ).toBeNull();
    const emptyDefs = context([]);
    emptyDefs.sourceCode.getScope = () => ({
      variables: [],
      upper: { variables: [{ name: "global", defs: [] }], upper: null },
    });
    expect(
      fetchInvocation(member({ type: "Literal", value: "fetch" }, ident("global")), emptyDefs),
    ).toHaveLength(1);
    const implicit = context([{ name: "self", defs: [{ type: "ImplicitGlobalVariable" }] }]);
    expect(
      fetchInvocation(member({ type: "Literal", value: "fetch" }, ident("self")), implicit),
    ).toHaveLength(1);
    expect(
      fetchInvocation(
        member(
          { type: "Literal", value: "fetch" },
          { type: "CallExpression", callee: ident("get") },
        ),
        globalContext,
      ),
    ).toBeNull();

    const sequence = {
      type: "CallExpression",
      callee: {
        type: "SequenceExpression",
        expressions: [{ type: "Literal", value: 0 }, fetchIdent],
      },
      arguments: [],
    };
    expect(fetchInvocation(sequence, globalContext)).toEqual([]);

    const spreadCall = {
      type: "CallExpression",
      callee: {
        type: "MemberExpression",
        property: ident("call"),
        object: fetchIdent,
      },
      arguments: [{ type: "SpreadElement" }],
    };
    expect(fetchInvocation(spreadCall, globalContext)).toBeNull();

    const apply = (elements, list = { type: "ArrayExpression", elements }) => ({
      type: "CallExpression",
      callee: { type: "MemberExpression", property: ident("apply"), object: fetchIdent },
      arguments: [{ type: "Literal", value: null }, list],
    });
    expect(fetchInvocation(apply([{ type: "Literal", value: "/api" }]), globalContext)).toEqual([
      { type: "Literal", value: "/api" },
    ]);
    expect(fetchInvocation(apply([null]), globalContext)).toBeNull();
    expect(fetchInvocation(apply([{ type: "SpreadElement" }]), globalContext)).toBeNull();
    expect(
      fetchInvocation(
        apply([], {
          type: "ChainExpression",
          expression: { type: "ArrayExpression", elements: [] },
        }),
        globalContext,
      ),
    ).toEqual([]);
  });

  it("follows a const alias and a const destructure, and stops on unstable bindings", () => {
    const alias = variable("load", {
      type: "Variable",
      parent: { kind: "const" },
      node: { id: ident("load"), init: fetchIdent },
    });
    expect(
      isFetchCall(
        { type: "CallExpression", callee: ident("load"), arguments: [] },
        context([alias]),
      ),
    ).toBe(true);

    const nested = {
      type: "ObjectPattern",
      properties: [
        { type: "RestElement" },
        {
          type: "Property",
          key: { type: "Literal", value: "other" },
          value: ident("nope"),
        },
        {
          type: "Property",
          key: ident("fetch"),
          value: { type: "AssignmentPattern", left: ident("load") },
        },
      ],
    };
    const destructured = variable("load", {
      type: "Variable",
      parent: { kind: "const" },
      node: { id: nested, init: ident("globalThis") },
    });
    expect(
      isFetchCall(
        { type: "CallExpression", callee: ident("load"), arguments: [] },
        context([destructured]),
      ),
    ).toBe(true);

    const dynamicKey = variable("schedule", {
      type: "Variable",
      parent: { kind: "const" },
      node: {
        id: {
          type: "ObjectPattern",
          properties: [
            {
              type: "Property",
              computed: true,
              key: ident("fetch"),
              value: ident("schedule"),
            },
          ],
        },
        init: ident("globalThis"),
      },
    });
    expect(
      isFetchCall(
        { type: "CallExpression", callee: ident("schedule"), arguments: [] },
        context([dynamicKey]),
      ),
    ).toBe(false);

    function keyed(name, key) {
      return variable(name, {
        type: "Variable",
        parent: { kind: "const" },
        node: {
          id: {
            type: "ObjectPattern",
            properties: [{ type: "Property", computed: true, key, value: ident(name) }],
          },
          init: ident("globalThis"),
        },
      });
    }
    const call = (name) => ({ type: "CallExpression", callee: ident(name), arguments: [] });
    expect(
      isFetchCall(
        call("fromLiteral"),
        context([keyed("fromLiteral", { type: "Literal", value: "fetch" })]),
      ),
    ).toBe(true);
    expect(
      isFetchCall(
        call("fromTemplate"),
        context([
          keyed("fromTemplate", {
            type: "TemplateLiteral",
            expressions: [],
            quasis: [{ value: { cooked: "fetch", raw: "fetch" } }],
          }),
        ]),
      ),
    ).toBe(true);

    const mutable = variable("load", {
      type: "Variable",
      parent: { kind: "let" },
      node: { id: ident("load"), init: fetchIdent },
    });
    expect(
      isFetchCall(
        { type: "CallExpression", callee: ident("load"), arguments: [] },
        context([mutable]),
      ),
    ).toBe(false);

    const left = variable("left", {
      type: "Variable",
      parent: { kind: "const" },
      node: { id: ident("left"), init: ident("right") },
    });
    const right = variable("right", {
      type: "Variable",
      parent: { kind: "const" },
      node: { id: ident("right"), init: ident("left") },
    });
    expect(
      isFetchCall(
        { type: "CallExpression", callee: ident("left"), arguments: [] },
        context([left, right]),
      ),
    ).toBe(false);

    const parameter = variable("load", { type: "Parameter", parent: { kind: "const" }, node: {} });
    expect(
      isFetchCall(
        { type: "CallExpression", callee: ident("load"), arguments: [] },
        context([parameter]),
      ),
    ).toBe(false);
    const arrayPattern = variable("load", {
      type: "Variable",
      parent: { kind: "const" },
      node: { id: { type: "ArrayPattern", elements: [] }, init: fetchIdent },
    });
    expect(
      isFetchCall(
        { type: "CallExpression", callee: ident("load"), arguments: [] },
        context([arrayPattern]),
      ),
    ).toBe(false);
    const nestedValue = variable("load", {
      type: "Variable",
      parent: { kind: "const" },
      node: {
        id: {
          type: "ObjectPattern",
          properties: [
            {
              type: "Property",
              key: ident("fetch"),
              value: { type: "ObjectPattern", properties: [] },
            },
          ],
        },
        init: ident("globalThis"),
      },
    });
    expect(
      isFetchCall(
        { type: "CallExpression", callee: ident("load"), arguments: [] },
        context([nestedValue]),
      ),
    ).toBe(false);
    const ambiguous = {
      name: "load",
      defs: [
        {
          type: "Variable",
          parent: { kind: "const" },
          node: { id: ident("load"), init: fetchIdent },
        },
        {
          type: "Variable",
          parent: { kind: "const" },
          node: { id: ident("load"), init: fetchIdent },
        },
      ],
    };
    expect(
      isFetchCall(
        { type: "CallExpression", callee: ident("load"), arguments: [] },
        context([ambiguous]),
      ),
    ).toBe(false);
  });
});

describe("logical call forwarding", () => {
  it("rewrites a literal apply array and rejects an opaque one", () => {
    const apply = (elements) =>
      logicalCall({
        type: "CallExpression",
        callee: { type: "MemberExpression", property: ident("apply"), object: ident("execFile") },
        arguments: [
          { type: "Literal", value: null },
          { type: "ArrayExpression", elements },
        ],
      });
    expect(apply([{ type: "Literal", value: "gh" }]).arguments).toEqual([
      { type: "Literal", value: "gh" },
    ]);
    expect(apply([null])).toBeNull();
    expect(apply([{ type: "SpreadElement" }])).toBeNull();
    expect(
      logicalCall({
        type: "CallExpression",
        callee: { type: "MemberExpression", property: ident("apply"), object: ident("execFile") },
        arguments: [
          { type: "Literal", value: null },
          {
            type: "ChainExpression",
            expression: { type: "ArrayExpression", elements: [{ type: "Literal", value: "gh" }] },
          },
        ],
      }).arguments,
    ).toEqual([{ type: "Literal", value: "gh" }]);
    expect(
      logicalCall({
        type: "CallExpression",
        callee: {
          type: "TSNonNullExpression",
          expression: {
            type: "MemberExpression",
            property: ident("catch"),
            object: ident("work"),
          },
        },
        arguments: [],
      }).callee.property.name,
    ).toBe("catch");
    expect(memberPropertyName({ type: "Identifier", name: "mock" })).toBeNull();
    expect(
      memberPropertyName({
        type: "MemberExpression",
        computed: true,
        property: {
          type: "TemplateLiteral",
          expressions: [],
          quasis: [{ value: { cooked: null, raw: "mock" } }],
        },
      }),
    ).toEqual("mock");
  });
});

describe("framework member names", () => {
  it("reads an expression-free template member and ignores other nodes", () => {
    expect(
      expressionName({
        type: "MemberExpression",
        computed: true,
        object: ident("vi"),
        property: {
          type: "TemplateLiteral",
          expressions: [],
          quasis: [{ value: { cooked: null, raw: "mock" } }],
        },
      }),
    ).toEqual("vi.mock");
    expect(expressionName(null)).toBeNull();
    expect(expressionName({ type: "Literal", value: 1 })).toBeNull();
    expect(
      expressionName({
        type: "ChainExpression",
        expression: {
          type: "MemberExpression",
          computed: true,
          object: ident("vi"),
          property: {
            type: "TemplateLiteral",
            expressions: [{ type: "Identifier", name: "name" }],
            quasis: [{ value: { cooked: "", raw: "" } }, { value: { cooked: "", raw: "" } }],
          },
        },
      }),
    ).toBeNull();
    expect(
      expressionName({
        type: "MemberExpression",
        computed: true,
        object: ident("vi"),
        property: {
          type: "TemplateLiteral",
          expressions: [],
          quasis: [{ value: { cooked: "mock", raw: "mock" } }],
        },
      }),
    ).toEqual("vi.mock");
  });
});
