import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";
import ts from "typescript";

const source = readFileSync(new URL("../src/App.tsx", import.meta.url), "utf8");
const tree = ts.createSourceFile("App.tsx", source, ts.ScriptTarget.Latest, true, ts.ScriptKind.TSX);
function find(predicate, node = tree) {
  if (predicate(node)) return node;
  return ts.forEachChild(node, child => find(predicate, child));
}
function run(text, state) {
  const js = ts.transpile(text, { target: ts.ScriptTarget.ES2022 });
  return vm.runInNewContext(js, state);
}
function call(name, state) {
  const node = find(n => ts.isFunctionDeclaration(n) && n.name?.text === name);
  assert.ok(node, name);
  return run(`${node.getText(tree)}; ${name}();`, state);
}
function context() {
  const calls = [];
  const state = {
    insurerFromDate: "2026-04-01",
    insurerOverview: { members: [{ rowId: 3 }, { rowId: 4 }] },
    setInsurerOverview(value) { state.insurerOverview = value; },
    setInsurerFromDate(value) { state.insurerFromDate = value; },
    setInsurerLoading() {}, setError() {}, setNotice() {},
    async invoke(command, args) { calls.push({ command, args }); return state.insurerOverview; },
  };
  return { state, calls };
}

test("changing the start date invalidates the displayed rows and export", () => {
  const input = find(n => ts.isJsxSelfClosingElement(n) && n.tagName.getText(tree) === "input"
    && n.attributes.properties.some(p => ts.isJsxAttribute(p) && p.name.getText(tree) === "value"
      && p.initializer?.getText(tree) === "{insurerFromDate}"));
  const handler = input.attributes.properties.find(p => p.name?.getText(tree) === "onChange");
  const { state } = context();
  state.event = { target: { value: "2026-05-15" } };
  run(`(${handler.initializer.expression.getText(tree)})(event);`, state);
  assert.equal(state.insurerFromDate, "2026-05-15");
  assert.equal(state.insurerOverview, null);
});

test("overview sends the exact selected calendar date", async () => {
  const { state, calls } = context();
  await call("loadInsurerOverview", state);
  assert.equal(calls[0].command, "get_insurer_overview");
  assert.equal(calls[0].args.fromDate, "2026-04-01");
});

test("failed reload does not leave stale exportable data", async () => {
  const { state } = context();
  state.invoke = async () => { throw new Error("test failure"); };
  await call("loadInsurerOverview", state);
  assert.equal(state.insurerOverview, null);
});

test("PDF sends the same date and exact visible record IDs", async () => {
  const { state, calls } = context();
  await call("exportInsurerOverviewPdf", state);
  assert.equal(calls[0].command, "export_insurer_overview_pdf");
  assert.equal(calls[0].args.fromDate, "2026-04-01");
  assert.deepEqual(Array.from(calls[0].args.expectedRowIds), [3, 4]);
  state.insurerOverview = null;
  await call("exportInsurerOverviewPdf", state);
  assert.equal(calls.length, 1);
});

test("claims list omits only the type column; detail and form retain description", () => {
  const start = source.indexOf('{visibleClaims.map');
  const listStart = source.lastIndexOf('<div className="claims-table">', start);
  const end = source.indexOf('</tbody></table></div>', start);
  const list = source.slice(listStart, end);
  const headers = [...list.matchAll(/<th>(.*?)<\/th>/g)].map(match => match[1]);
  assert.deepEqual(headers, ["Číslo události", "Člen", "Evidenční číslo", "Kód OC", "Datum vzniku", "Datum nahlášení", "Stav", "Požadovaná částka", "Vyplacená částka", "Poslední změna", "Akce"]);
  assert.ok(!list.includes("display(claim.description)"));
  assert.ok(list.includes('colSpan={11}'));
  assert.ok(source.slice(end).includes("display(claim.description)"));
  assert.ok(source.includes('value={claimForm.description}'));
  assert.ok(list.includes("editAgendaClaim(claim)"));
});
