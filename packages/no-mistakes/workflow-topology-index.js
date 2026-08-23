"use strict";

// A pure-JS query index rebuilt from the `ciTopology()` JSON, ported from
// the original engine's `topology-index.mts` + `frozen-topology.mts`. This
// stays JS-only by design: it returns closures over frozen `Map`s, which
// doesn't cross the N-API boundary cleanly, and the native side already
// does the expensive parse/diagnose/resolve work — this index is just
// deterministic, sorted traversal over data it's handed.

function deepFrozenClone(value) {
  return freezeRecursively(structuredClone(value));
}

function freezeRecursively(value) {
  if (value === null || typeof value !== "object") return value;
  for (const nested of Object.values(value)) freezeRecursively(nested);
  return Object.freeze(value);
}

// `Object.freeze(new Map())` does NOT stop `.set()`/`.delete()` from
// working (freeze only covers own enumerable properties, not a Map's
// internal slots) — this wrapper is what actually makes `workflowsByPath`
// / `jobsById` read-only.
class FrozenReadonlyMap {
  #source;

  constructor(source) {
    this.#source = source;
    Object.freeze(this);
  }

  get size() {
    return this.#source.size;
  }

  entries() {
    return this.#source.entries();
  }

  forEach(callback, thisArg) {
    for (const [key, value] of this.#source) callback.call(thisArg, value, key, this);
  }

  get(key) {
    return this.#source.get(key);
  }

  has(key) {
    return this.#source.has(key);
  }

  keys() {
    return this.#source.keys();
  }

  values() {
    return this.#source.values();
  }

  [Symbol.iterator]() {
    return this.entries();
  }

  get [Symbol.toStringTag]() {
    return "Map";
  }
}

function adjacencyMap(ids) {
  return new Map([...ids].map((id) => [id, new Set()]));
}

function artifactEdgeMap(ids) {
  return new Map([...ids].map((id) => [id, []]));
}

function assertKnown(values, id, label) {
  if (!values.has(id)) throw new Error(`unknown ${label}: ${id}`);
}

function query(adjacency, source, transitive) {
  const direct = adjacency.get(source);
  if (!transitive) return Object.freeze([...direct].sort());
  const visited = new Set();
  const pending = [...direct];
  while (pending.length > 0) {
    const current = pending.pop();
    if (current === source || visited.has(current)) continue;
    visited.add(current);
    for (const neighbor of adjacency.get(current)) {
      if (neighbor !== source && !visited.has(neighbor)) pending.push(neighbor);
    }
  }
  return Object.freeze([...visited].sort());
}

function workflowPathFromId(id) {
  const index = id.indexOf("#");
  return index === -1 ? id : id.slice(0, index);
}

function artifactEdgeKey(edge) {
  return [edge.name, edge.from, edge.to, edge.producerStep, edge.consumerStep, edge.match].join(
    "\0",
  );
}

function workflowRunEdgeKey(edge) {
  const metadata = {
    types: edge.types,
    branches: edge.branches,
    branchesIgnore: edge.branchesIgnore,
  };
  return [edge.from, edge.to, JSON.stringify(metadata)].join("\0");
}

/**
 * Builds a frozen, sorted query index over a `WorkflowTopology` (the
 * parsed object returned by `ciTopology()`, or `JSON.parse`d from
 * `no-mistakes ci topology --format json`).
 */
function createWorkflowTopologyIndex(topology) {
  const workflowsByPath = new Map(
    topology.workflows.map((workflow) => [workflow.path, deepFrozenClone(workflow)]),
  );
  const jobsById = new Map(topology.jobs.map((job) => [job.id, deepFrozenClone(job)]));
  const upstreamJobs = adjacencyMap(jobsById.keys());
  const downstreamJobs = adjacencyMap(jobsById.keys());
  const callerJobs = adjacencyMap(workflowsByPath.keys());
  const callers = adjacencyMap(workflowsByPath.keys());
  const callees = adjacencyMap(workflowsByPath.keys());
  const workflowRunSources = adjacencyMap(workflowsByPath.keys());
  const workflowRunSubscribers = adjacencyMap(workflowsByPath.keys());
  const artifactProducers = artifactEdgeMap(jobsById.keys());
  const artifactConsumers = artifactEdgeMap(jobsById.keys());
  const incomingWorkflowRunEdges = new Map([...workflowsByPath.keys()].map((path) => [path, []]));
  const outgoingWorkflowRunEdges = new Map([...workflowsByPath.keys()].map((path) => [path, []]));

  for (const edge of topology.edges) {
    if (edge.kind === "needs") {
      if (!jobsById.has(edge.from) || !jobsById.has(edge.to)) continue;
      upstreamJobs.get(edge.to).add(edge.from);
      downstreamJobs.get(edge.from).add(edge.to);
      continue;
    }
    if (edge.kind === "workflow-run") {
      if (!workflowsByPath.has(edge.from) || !workflowsByPath.has(edge.to)) continue;
      const edgeSnapshot = deepFrozenClone(edge);
      workflowRunSources.get(edge.to).add(edge.from);
      workflowRunSubscribers.get(edge.from).add(edge.to);
      incomingWorkflowRunEdges.get(edge.to).push(edgeSnapshot);
      outgoingWorkflowRunEdges.get(edge.from).push(edgeSnapshot);
      continue;
    }
    if (edge.kind === "artifact") {
      if (!jobsById.has(edge.from) || !jobsById.has(edge.to)) continue;
      const edgeSnapshot = deepFrozenClone(edge);
      artifactProducers.get(edge.to).push(edgeSnapshot);
      artifactConsumers.get(edge.from).push(edgeSnapshot);
      continue;
    }
    // edge.kind === "calls"
    if (!edge.local || !edge.to || !jobsById.has(edge.from) || !workflowsByPath.has(edge.to))
      continue;
    const callerPath = workflowPathFromId(edge.from);
    if (!workflowsByPath.has(callerPath)) continue;
    callerJobs.get(edge.to).add(edge.from);
    callers.get(edge.to).add(callerPath);
    callees.get(callerPath).add(edge.to);
  }

  const jobQuery = (adjacency, transitive) => (jobId) => {
    assertKnown(jobsById, jobId, "workflow job");
    return query(adjacency, jobId, transitive);
  };
  const workflowQuery = (adjacency, transitive) => (workflowPath) => {
    assertKnown(workflowsByPath, workflowPath, "workflow");
    return query(adjacency, workflowPath, transitive);
  };
  const workflowRunEdgeQuery = (edgesByPath) => (workflowPath) => {
    assertKnown(workflowsByPath, workflowPath, "workflow");
    return Object.freeze(
      [...edgesByPath.get(workflowPath)].sort((left, right) =>
        workflowRunEdgeKey(left).localeCompare(workflowRunEdgeKey(right)),
      ),
    );
  };
  const artifactEdgeQuery = (edgesByJob) => (jobId) => {
    assertKnown(jobsById, jobId, "workflow job");
    return Object.freeze(
      [...edgesByJob.get(jobId)].sort((left, right) =>
        artifactEdgeKey(left).localeCompare(artifactEdgeKey(right)),
      ),
    );
  };

  return Object.freeze({
    workflowsByPath: new FrozenReadonlyMap(workflowsByPath),
    jobsById: new FrozenReadonlyMap(jobsById),
    directUpstreamJobIds: jobQuery(upstreamJobs, false),
    transitiveUpstreamJobIds: jobQuery(upstreamJobs, true),
    directDownstreamJobIds: jobQuery(downstreamJobs, false),
    transitiveDownstreamJobIds: jobQuery(downstreamJobs, true),
    directCallerJobIds: workflowQuery(callerJobs, false),
    directCallerWorkflowPaths: workflowQuery(callers, false),
    transitiveCallerWorkflowPaths: workflowQuery(callers, true),
    directCalleeWorkflowPaths: workflowQuery(callees, false),
    transitiveCalleeWorkflowPaths: workflowQuery(callees, true),
    incomingWorkflowRunEdges: workflowRunEdgeQuery(incomingWorkflowRunEdges),
    outgoingWorkflowRunEdges: workflowRunEdgeQuery(outgoingWorkflowRunEdges),
    directWorkflowRunSourcePaths: workflowQuery(workflowRunSources, false),
    transitiveWorkflowRunSourcePaths: workflowQuery(workflowRunSources, true),
    directWorkflowRunSubscriberPaths: workflowQuery(workflowRunSubscribers, false),
    transitiveWorkflowRunSubscriberPaths: workflowQuery(workflowRunSubscribers, true),
    artifactProducersForConsumerJob: artifactEdgeQuery(artifactProducers),
    artifactConsumersForProducerJob: artifactEdgeQuery(artifactConsumers),
  });
}

module.exports = { createWorkflowTopologyIndex };
