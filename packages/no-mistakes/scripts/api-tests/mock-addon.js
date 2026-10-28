function createMockAddon(collectionGap) {
  const queryResult = (command, json) => {
    const options = JSON.parse(json);
    return JSON.stringify({
      command,
      options,
      ...(options.file === "__partial__" ? collectionGap : {}),
    });
  };
  return {
    dependenciesJson: async (json) => {
      const options = JSON.parse(json);
      if (options.root === "__locked__") {
        throw new Error("another no-mistakes invocation holds the lock");
      }
      return JSON.stringify({ command: "dependencies", options });
    },
    dependentsJson: async (json) =>
      JSON.stringify({ command: "dependents", options: JSON.parse(json) }),
    relatedJson: async (json) => JSON.stringify({ command: "related", options: JSON.parse(json) }),
    analyzeProjectJson: async (json) =>
      JSON.stringify({ command: "analyzeProject", options: JSON.parse(json) }),
    symbolsJson: async (json) => JSON.stringify({ command: "symbols", options: JSON.parse(json) }),
    importUsagesJson: async (json) =>
      JSON.stringify({ command: "importUsages", options: JSON.parse(json) }),
    importersJson: async (json) => queryResult("importers", json),
    exportsOfJson: async (json) => queryResult("exportsOf", json),
    deadExportsJson: async (json) => queryResult("deadExports", json),
    callSitesJson: async (json) => queryResult("callSites", json),
    resolveCheckJson: async (json) => {
      const options = JSON.parse(json);
      if (Array.isArray(options.files) && options.files.length === 0) {
        throw new Error("files must contain at least one path");
      }
      if (Object.hasOwn(options, "file") && Object.hasOwn(options, "files")) {
        throw new Error("exactly one of file or files is required");
      }
      return JSON.stringify({ command: "resolveCheck", options });
    },
    fetchesJson: async (json) => JSON.stringify({ command: "fetches", options: JSON.parse(json) }),
    checkJson: async (json) => {
      const options = JSON.parse(json);
      return JSON.stringify({
        command: "check",
        options,
        ...(options.jobs === 2
          ? {
              jobsPoolWarning: {
                code: "rayon-pool-size",
                message: "existing pool",
                requestedJobs: 2,
                effectiveJobs: 4,
              },
            }
          : {}),
      });
    },
    lockStatusJson: async (json) =>
      JSON.stringify({ command: "lockStatus", options: JSON.parse(json) }),
    resolveConfigJson: async (json) =>
      JSON.stringify({ command: "resolveConfig", options: JSON.parse(json) }),
    validateMermaidMarkdownJson: async (json) =>
      JSON.stringify({ command: "validateMermaidMarkdown", options: JSON.parse(json) }),
    testsPlanJson: async (json) =>
      JSON.stringify({ command: "testsPlan", options: JSON.parse(json) }),
    testsTargetsJson: async (json) =>
      JSON.stringify({ command: "testsTargets", options: JSON.parse(json) }),
    testsWhyJson: async (json) =>
      JSON.stringify({ command: "testsWhy", options: JSON.parse(json) }),
    testsCommentMarkdown: async (json) =>
      `comment:${JSON.parse(json).plan || JSON.parse(json).planJson?.selected_tests?.length}`,
    testsGraphJson: async (json) =>
      JSON.stringify({ command: "testsGraph", options: JSON.parse(json) }),
    testsGraphMermaid: async (json) =>
      `graph:${JSON.parse(json).plan || JSON.parse(json).planJson?.selected_tests?.length}`,
    playwrightCheckJson: async (json) =>
      JSON.stringify({ command: "playwrightCheck", options: JSON.parse(json) }),
    playwrightEdgesJson: async (json) =>
      JSON.stringify({ command: "playwrightEdges", options: JSON.parse(json) }),
    playwrightRelatedJson: async (json) =>
      JSON.stringify({ command: "playwrightRelated", options: JSON.parse(json) }),
    playwrightTestsJson: async (json) =>
      JSON.stringify({ command: "playwrightTests", options: JSON.parse(json) }),
    queuesJson: async (json) => JSON.stringify({ command: "queues", options: JSON.parse(json) }),
    queueEdgesJson: async (json) =>
      JSON.stringify({ command: "queueEdges", options: JSON.parse(json) }),
    queueRelatedJson: async (json) =>
      JSON.stringify({ command: "queueRelated", options: JSON.parse(json) }),
    queueCheckJson: async (json) =>
      JSON.stringify({ command: "queueCheck", options: JSON.parse(json) }),
    serverRoutesJson: async (json) =>
      JSON.stringify({ command: "serverRoutes", options: JSON.parse(json) }),
    serverRouteListJson: async (json) =>
      JSON.stringify({ command: "serverRouteList", options: JSON.parse(json) }),
    serverRouteEdgesJson: async (json) =>
      JSON.stringify({ command: "serverRouteEdges", options: JSON.parse(json) }),
    serverRouteRelatedJson: async (json) =>
      JSON.stringify({ command: "serverRouteRelated", options: JSON.parse(json) }),
    serverContractsJson: async (json) =>
      JSON.stringify({ command: "serverContracts", options: JSON.parse(json) }),
    flowJson: async (json) => JSON.stringify({ command: "flow", options: JSON.parse(json) }),
    reactAnalyzeJson: async (json) =>
      JSON.stringify({ command: "reactAnalyze", options: JSON.parse(json) }),
    reactCheckJson: async (json) =>
      JSON.stringify({ command: "reactCheck", options: JSON.parse(json) }),
    reactUsagesJson: async (json) =>
      JSON.stringify({ command: "reactUsages", options: JSON.parse(json) }),
    infraResourceRefsJson: async (json) =>
      JSON.stringify({ command: "infraResourceRefs", options: JSON.parse(json) }),
    parsePostgresSqlJson: async (json) =>
      JSON.stringify({ command: "parsePostgresSql", options: JSON.parse(json) }),
    extractExecutedPostgresSqlJson: async (json) =>
      JSON.stringify(
        JSON.parse(json).includeDiagnostics === true
          ? {
              statements: [],
              undeclaredExecutorTypes: [{ type: "PoolClient", module: "@db", count: 1 }],
              unresolvedExecutorReceivers: [{ type: "Pool", module: "@db", count: 2 }],
            }
          : { command: "extractExecutedPostgresSql", options: JSON.parse(json) },
      ),
    generatePostgresCatalogJson: async (json) =>
      JSON.stringify({ command: "generatePostgresCatalog", options: JSON.parse(json) }),
    infraOutputsJson: async (json) =>
      JSON.stringify({ command: "infraOutputs", options: JSON.parse(json) }),
    infraTestForJson: async (json) =>
      JSON.stringify({ command: "infraTestFor", options: JSON.parse(json) }),
    swiftImportersJson: async (json) =>
      JSON.stringify({ command: "swiftImporters", options: JSON.parse(json) }),
    swiftTestTargetsJson: async (json) =>
      JSON.stringify({ command: "swiftTestTargets", options: JSON.parse(json) }),
    ciTopologyJson: async (json) =>
      JSON.stringify({ command: "ciTopology", options: JSON.parse(json) }),
    ciTopologyImpactJson: async (json) =>
      JSON.stringify({ command: "ciTopologyImpact", options: JSON.parse(json) }),
    version: async () => "1.2.3",
  };
}

module.exports = { createMockAddon };
