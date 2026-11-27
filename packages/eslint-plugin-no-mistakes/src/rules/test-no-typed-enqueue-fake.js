"use strict";

const { posix } = require("node:path");
const { options, rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { isTransparentExpression, unwrapExpression } = require("./async-ast");
const { isUnresolved, moduleLoad } = require("./module-load");
const { reportUnanalyzable, UNANALYZABLE_SCHEMA } = require("./rule-unanalyzable");
const { memberPropertyName } = require("./logical-call");
const {
  isFrameworkMockType,
  isFrameworkReceiver,
  referenceBinding,
  resolveImportTarget,
  typeArgParams,
} = require("./test-no-typed-enqueue-fake-helpers");

const id = "test-no-typed-enqueue-fake";

function exportPattern(pattern) {
  if (pattern === undefined) return null;
  try {
    return new RegExp(pattern);
  } catch (cause) {
    throw new Error(`${id}: singleExportPattern must be a valid regular expression`, { cause });
  }
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "require enqueue fakes to declare their argument tuple explicitly",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: {
          singleExportPattern: { type: "string", pattern: "\\S" },
          moduleGlobs: { type: "array", items: { type: "string", pattern: "\\S" } },
          unanalyzable: UNANALYZABLE_SCHEMA,
        },
        additionalProperties: false,
      },
    ],
    messages: {
      typedEnqueueFake:
        "Type the enqueue fake with an explicit argument tuple, or use the real function with a mocked queue.",
      unanalyzable:
        "Cannot tell whether {{subject}} is an enqueue export. Use a static import or require.",
    },
  },
  (context) => {
    const { singleExportPattern, moduleGlobs = [] } = options(context);
    const exportMatcher = exportPattern(singleExportPattern);
    if (!exportMatcher && moduleGlobs.length === 0) return {};
    const { sourceCode } = context;
    const suppressed = ruleSuppression(context, id);

    function matchesTarget(node) {
      const target = resolveImportTarget(node, sourceCode);
      return (
        Boolean(target) &&
        (!exportMatcher || exportMatcher.test(target.name)) &&
        (moduleGlobs.length === 0 ||
          (typeof target.module === "string" &&
            moduleGlobs.some((glob) => posix.matchesGlob(target.module, glob))))
      );
    }

    function matchesParameters(typeNode) {
      const typeName = typeNode?.type === "TSTypeReference" ? typeNode.typeName : null;
      if (
        typeName?.type !== "Identifier" ||
        typeName.name !== "Parameters" ||
        referenceBinding(typeName, sourceCode)?.defs.length
      ) {
        return false;
      }
      const args = typeArgParams(typeNode);
      return (
        args?.length === 1 && args[0]?.type === "TSTypeQuery" && matchesTarget(args[0].exprName)
      );
    }

    function unresolvedTypedTarget(params, lintContext) {
      return Boolean(params?.some((param) => typeQueryUnresolved(param, lintContext)));
    }

    function typeQueryName(expr) {
      let current = expr;
      while (current?.type === "TSQualifiedName") current = current.left;
      return current;
    }

    function typeQueryUnresolved(typeNode, lintContext) {
      if (typeNode?.type === "TSTypeQuery") {
        return isUnresolved(moduleLoad(typeQueryName(typeNode.exprName), lintContext));
      }
      if (typeNode?.type !== "TSTypeReference") return false;
      return unresolvedTypedTarget(typeArgParams(typeNode), lintContext);
    }

    function hasTypedEnqueueArg(params) {
      return Boolean(
        params?.some(
          (param) =>
            (param?.type === "TSTypeQuery" && matchesTarget(param.exprName)) ||
            matchesParameters(param) ||
            (param?.type === "TSFunctionType" &&
              param.params.some((arg) => matchesParameters(arg.typeAnnotation?.typeAnnotation))),
        ),
      );
    }

    function report(node) {
      if (!suppressed(node)) context.report({ node, messageId: "typedEnqueueFake" });
    }

    return {
      CallExpression(node) {
        let callee = node.callee;
        let typeNode = node;
        while (isTransparentExpression(callee)) {
          if (callee.type === "TSInstantiationExpression") typeNode = callee;
          callee = callee.expression;
        }
        if (
          callee?.type !== "MemberExpression" ||
          !isFrameworkReceiver(callee.object, sourceCode)
        ) {
          return;
        }
        const method = memberPropertyName(callee);
        const argument = unwrapExpression(node.arguments[0]);
        if (method === "fn" && hasTypedEnqueueArg(typeArgParams(typeNode))) {
          report(node);
        } else if (method === "mocked" && matchesTarget(argument)) {
          report(node);
        } else if (
          (method === "mocked" || method === "fn") &&
          !suppressed(node) &&
          (isUnresolved(moduleLoad(method === "mocked" ? argument : callee, context)) ||
            (method === "fn" && unresolvedTypedTarget(typeArgParams(typeNode), context)))
        ) {
          reportUnanalyzable(context, node, "this enqueue fake");
        }
      },
      TSTypeReference(node) {
        if (
          isFrameworkMockType(node.typeName, sourceCode) &&
          hasTypedEnqueueArg(typeArgParams(node))
        ) {
          report(node);
        }
      },
    };
  },
);
