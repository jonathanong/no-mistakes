"use strict";

const { isModuleMutable, isResetAssignment } = require("./test-no-shared-state-analysis");
const {
  collectPatternNames,
  isMutableInitializer,
  mutatingCallPropertyName,
  mutatingCallTarget,
  mutationPath,
  mutationRootName,
} = require("./test-no-shared-state-helpers");

function createMutationHandlers({
  cleanupTracker,
  context,
  depths,
  mutableTopLevel,
  registryReports,
  viMockTracker,
}) {
  function reportIfShared(node, name) {
    if (
      name &&
      depths.test() > 0 &&
      depths.setup() === 0 &&
      !viMockTracker.isCaptured(name) &&
      isModuleMutable({ context, mutableTopLevel, node, name })
    ) {
      context.report({ node, messageId: "shared" });
    }
  }

  function reportAssignment(node) {
    for (const name of collectPatternNames(node.left)) reportIfShared(node, name);
    if (node.left.type !== "MemberExpression") return;
    reportIfShared(node, mutationRootName(node.left));
  }

  function rememberSetupCleanup(node, name, path) {
    if (
      name &&
      depths.setup() > 0 &&
      mutableTopLevel.has(name) &&
      isModuleMutable({ context, mutableTopLevel, node, name })
    ) {
      cleanupTracker.remember(path);
      cleanupTracker.rememberPendingBeforeAll(path);
    }
  }

  function rememberCall(node) {
    const { name, path } = mutatingCallTarget(node);
    if (mutatingCallPropertyName(node) === "clear") {
      rememberSetupCleanup(node, name, path);
    }
    registryReports.remember(node, name, path, depths.test(), depths.setup());
  }

  function rememberAssignmentCleanup(node) {
    if (depths.setup() === 0) return;
    if (!isResetAssignment(node, isMutableInitializer)) return;
    if (node.left.type === "Identifier") {
      rememberSetupCleanup(node, node.left.name, node.left.name);
      return;
    }
    rememberSetupCleanup(node, mutationRootName(node.left), mutationPath(node.left.object));
  }

  return {
    mutationWalk: {
      onAssignment: (assignment) => {
        rememberAssignmentCleanup(assignment);
        reportAssignment(assignment);
      },
      onCall: rememberCall,
      onUpdate: (update) => reportIfShared(update, mutationRootName(update.argument)),
    },
    rememberAssignmentCleanup,
    rememberCall,
    reportAssignment,
    reportIfShared,
  };
}

module.exports = {
  createMutationHandlers,
};
