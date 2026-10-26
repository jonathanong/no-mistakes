const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const queryLoopRoot = join(
  __dirname,
  "../../../test-cases/rules/postgres-ddl-unanalyzable/fixture/query-loops",
);
const sql = readFileSync(
  join(__dirname, "../../../fixtures/postgres-facts/source/procedural-loop-coverage.sql"),
  "utf8",
);

function shape(occurrences) {
  return occurrences.map(({ kind, occurrences: children }) =>
    children.length ? `${kind}(${shape(children).join(",")})` : kind,
  );
}

for (const [config, findings, advisories] of [
  ["valid.yml", 0, 0],
  ["valid-report.yml", 0, 0],
  ["default.yml", 0, 11],
  ["report.yml", 11, 0],
  ["generated-write.yml", 1, 0],
]) {
  test(
    `compiled query-loop check preserves policy and prepared parity: ${config}`,
    { skip: !compiled },
    async () => {
      const cjs = require("../index.js");
      const esm = await import("../index.mjs");
      const options = { root: queryLoopRoot, config };
      const pending = cjs.check(options);
      assert.equal(typeof pending.then, "function");
      const report = await pending;
      assert.deepEqual(await esm.check(options), report);
      const prepared = await cjs.analyzeProject({ ...options, reports: [{ type: "check" }] });
      assert.deepEqual(prepared.reports[0].result, report);
      assert.equal(report.rules.length, findings);
      assert.equal(report.advisories.length, advisories);
      if (config === "generated-write.yml") {
        assert.equal(report.rules[0].rule, "postgres-no-generated-column-writes");
        assert.match(report.rules[0].message, /do not write generated column/);
      } else {
        for (const row of [...report.rules, ...report.advisories]) {
          assert.equal(row.file, "malformed.sql");
          assert.equal(row.line, 6);
        }
      }
    },
  );
}

test(
  "compiled CJS and ESM procedural completeness requires typed utility coverage",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const input = { sql, fileName: "procedural-loop-coverage.sql" };
    const pending = cjs.parsePostgresSql(input);
    assert.equal(typeof pending.then, "function");
    const facts = await pending;
    assert.deepEqual(await esm.parsePostgresSql(input), facts);
    assert.deepEqual(await esm.parsePostgresSql([input]), [facts]);
    assert.deepEqual(facts.diagnostics, []);
    assert.equal(facts.statements.length, 21);
    const blocks = facts.statements.map((statement) => statement.block);
    for (const [index, block] of blocks.entries()) {
      assert.equal(block.complete, [16, 19, 20].includes(index), `block ${index}`);
      assert.equal(block.diagnostics.length === 0, block.complete, `block ${index}`);
    }
    const expected = [
      ["controlFlow(utility)"],
      ["controlFlow(utility)"],
      ["controlFlow(utility)"],
      ["controlFlow(controlFlow(utility))"],
      ["controlFlow(controlFlow(utility),utility)"],
      ["controlFlow(controlFlow(utility,utility))"],
      ["utility", "controlFlow(utility)", "utility"],
      ["controlFlow(utility)"],
    ];
    for (const [index, kinds] of expected.entries()) {
      assert.deepEqual(shape(blocks[index].occurrences), kinds, `block ${index}`);
    }
    assert.deepEqual(shape(blocks[10].occurrences), ["controlFlow(dml)"]);
    assert.deepEqual(shape(blocks[11].occurrences), ["controlFlow(utility)"]);
    assert.deepEqual(shape(blocks[12].occurrences), ["controlFlow(dynamicExecute)"]);
    assert.deepEqual(shape(blocks[13].occurrences), ["controlFlow(unknown)"]);
    assert.deepEqual(shape(blocks[14].occurrences), ["controlFlow(unknown)"]);
    assert.deepEqual(shape(blocks[15].occurrences), ["controlFlow(utility,controlFlow)"]);
    assert.deepEqual(shape(blocks[17].occurrences), ["controlFlow(utility)"]);
    const bytes = Buffer.from(sql);
    const slice = (span) => bytes.subarray(span.start.offset, span.end.offset).toString();
    for (const index of [0, 1, 2, 3, 5, 6, 7, 11, 15, 17]) {
      const notes = blocks[index].diagnostics.filter(({ message }) =>
        message.includes("no typed statement projection"),
      );
      assert.equal(notes.length, index === 5 ? 2 : 1, `block ${index}`);
      for (const note of notes) {
        assert.match(slice(note.span), /^(CREATE|EXECUTE|LOCK)/);
      }
    }
    assert.equal(blocks[0].statements.length, 0);
    assert.match(slice(blocks[0].diagnostics[0].span), /^CREATE TABLE bad /);
    assert.deepEqual(
      blocks[6].statements.map(({ kind }) => kind),
      ["createTable", "createTable"],
    );
    assert.match(blocks[6].statements[0].sql, /^CREATE TABLE before_loop/);
    assert.match(blocks[6].statements[1].sql, /^CREATE TABLE after_loop/);

    const conditional = blocks[16].statements[0];
    assert.equal(conditional.kind, "conditional");
    assert.equal(conditional.branches.length, 3);
    const names = ["branch_first", "branch_second", "branch_third"];
    for (const [index, branch] of conditional.branches.entries()) {
      assert.equal(branch.condition !== null, index < 2);
      assert.equal(branch.statements.length, 1);
      const statement = branch.statements[0];
      assert.equal(statement.kind, "createTable");
      assert.ok(statement.sql.includes(names[index]));
      assert.equal(slice(statement.span), statement.sql);
      assert.ok(branch.span.start.offset <= statement.span.start.offset);
      assert.ok(branch.span.end.offset >= statement.span.end.offset);
      if (index) {
        assert.ok(conditional.branches[index - 1].span.end.offset <= branch.span.start.offset);
      }
    }
    assert.equal(blocks[19].statements[0].kind, "literalExecute");
    assert.equal(blocks[19].statements[0].execute.statements[0].kind, "createTable");
    assert.equal(blocks[20].statements[0].kind, "doBlock");
    assert.equal(blocks[20].statements[0].block.statements[0].kind, "createTable");
  },
);
