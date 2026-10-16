// Checks the numbers of the quality bar (ADR 17) against tools/quality/thresholds.json.
//
//   node check.mjs coverage <llvm-cov.json>     line coverage per tier
//   node check.mjs mutation <crate> <outcomes>  mutation score of one crate
//   node check.mjs ratchet <base-ref>           no threshold is lower than on the base
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const thresholdsPath = join(here, "thresholds.json");
const thresholds = JSON.parse(readFileSync(thresholdsPath, "utf8"));
const repo = join(here, "..", "..");

const pct = (part, whole) => (whole === 0 ? 100 : (part / whole) * 100);
const fail = (message) => {
  console.error(message);
  process.exit(1);
};

function coverage(reportPath) {
  const files = JSON.parse(readFileSync(reportPath, "utf8")).data[0].files;
  const lines = files.map((f) => ({
    path: f.filename.slice(repo.length + 1),
    covered: f.summary.lines.covered,
    count: f.summary.lines.count,
  }));
  let failed = false;
  console.log("tier        lines   min");
  for (const [tier, { min, paths }] of Object.entries(thresholds.coverage)) {
    const mine = lines.filter((l) => paths.some((p) => l.path.startsWith(p)));
    if (mine.length === 0) fail(`No file in the coverage report belongs to tier "${tier}".`);
    const score = pct(
      mine.reduce((sum, l) => sum + l.covered, 0),
      mine.reduce((sum, l) => sum + l.count, 0),
    );
    const ok = score >= min;
    failed ||= !ok;
    console.log(`${tier.padEnd(10)} ${score.toFixed(1).padStart(5)}%  ${min}%  ${ok ? "ok" : "BELOW"}`);
  }
  if (failed) fail("Line coverage is below the threshold of a tier.");
}

// A mutant that timed out counts as caught: the tests noticed the change by hanging.
function mutation(crate, outcomesPath) {
  const min = thresholds.mutation[crate]?.min;
  if (min === undefined) fail(`No mutation threshold for ${crate}.`);
  const { caught, timeout, missed } = JSON.parse(readFileSync(outcomesPath, "utf8"));
  const score = pct(caught + timeout, caught + timeout + missed);
  console.log(`${crate}: ${score.toFixed(1)}% caught (${missed} missed, minimum ${min}%)`);
  if (score < min) fail(`Mutation score of ${crate} is below ${min}%.`);
}

function ratchet(baseRef) {
  let base;
  try {
    base = JSON.parse(
      execFileSync("git", ["show", `${baseRef}:tools/quality/thresholds.json`], { cwd: repo }).toString(),
    );
  } catch {
    console.log(`${baseRef} has no thresholds file yet.`);
    return;
  }
  const problems = [];
  for (const kind of ["coverage", "mutation"]) {
    for (const [name, { min }] of Object.entries(base[kind])) {
      const now = thresholds[kind][name]?.min;
      if (now === undefined) problems.push(`${kind} "${name}" was removed`);
      else if (now < min) problems.push(`${kind} "${name}" went down from ${min}% to ${now}%`);
    }
  }
  if (problems.length > 0) fail(`Thresholds can be raised but not lowered:\n  ${problems.join("\n  ")}`);
  console.log(`No threshold is lower than on ${baseRef}.`);
}

const [mode, ...args] = process.argv.slice(2);
if (mode === "coverage" && args.length === 1) coverage(args[0]);
else if (mode === "mutation" && args.length === 2) mutation(args[0], args[1]);
else if (mode === "ratchet" && args.length === 1) ratchet(args[0]);
else fail("Usage: check.mjs coverage <report> | mutation <crate> <outcomes> | ratchet <base-ref>");
