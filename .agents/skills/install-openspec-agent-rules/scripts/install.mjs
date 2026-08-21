#!/usr/bin/env node
/**
 * Install OpenSpec app-level agent rules into a project root.
 * Zero dependencies. Usage: node install.mjs --product Name --blurb "..." --stack "..." --paths "..."
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const SKILL_ROOT = path.resolve(__dirname, "..");
const TEMPLATES = path.join(SKILL_ROOT, "templates");

function parseArgs(argv) {
  const out = {
    root: process.cwd(),
    product: "",
    blurb: "",
    stack: "",
    paths: "",
    specsTable: "| （在 openspec/specs/ 落地后填写） | `openspec/specs/<capability>/` |",
    force: false,
    cursorRule: true,
  };
  for (let i = 2; i < argv.length; i++) {
    const a = argv[i];
    const next = () => argv[++i];
    if (a === "--root") out.root = path.resolve(next());
    else if (a === "--product") out.product = next();
    else if (a === "--blurb") out.blurb = next();
    else if (a === "--stack") out.stack = next();
    else if (a === "--paths") out.paths = next();
    else if (a === "--specs-table") out.specsTable = next();
    else if (a === "--force") out.force = true;
    else if (a === "--no-cursor-rule") out.cursorRule = false;
    else if (a === "--help" || a === "-h") out.help = true;
    else {
      console.error(`Unknown arg: ${a}`);
      process.exit(1);
    }
  }
  return out;
}

function render(tmpl, vars) {
  return tmpl.replace(/\{\{(\w+)\}\}/g, (_, key) => {
    if (!(key in vars)) throw new Error(`Missing template var: ${key}`);
    return vars[key];
  });
}

function ensureGitignoreCursor(root) {
  const gi = path.join(root, ".gitignore");
  const block = [
    "",
    "# Cursor: local only（rules/commands/skills 本机生成，不进 git）",
    ".cursor/",
    "",
  ].join("\n");

  if (!fs.existsSync(gi)) {
    fs.writeFileSync(gi, block.trimStart(), "utf8");
    return "created .gitignore with .cursor/";
  }

  const text = fs.readFileSync(gi, "utf8");
  // Already ignores .cursor somehow
  if (/(^|\/)\.cursor\/?\s*$/m.test(text) || /^\.cursor\/$/m.test(text)) {
    return "gitignore already ignores .cursor/";
  }

  fs.appendFileSync(gi, block, "utf8");
  return "appended .cursor/ to .gitignore";
}

function writeFile(filePath, content, force) {
  if (fs.existsSync(filePath) && !force) {
    return `skip (exists): ${filePath}`;
  }
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content, "utf8");
  return `wrote: ${filePath}`;
}

function main() {
  const args = parseArgs(process.argv);
  if (args.help) {
    console.log(`Usage:
  node install.mjs --product NAME --blurb "..." --stack "..." --paths "..."
    [--specs-table "..."] [--root DIR] [--force] [--no-cursor-rule]`);
    process.exit(0);
  }

  const missing = ["product", "blurb", "stack", "paths"].filter((k) => !args[k]);
  if (missing.length) {
    console.error(`Missing required: ${missing.map((m) => "--" + m).join(", ")}`);
    process.exit(1);
  }

  const vars = {
    PRODUCT_NAME: args.product,
    PRODUCT_BLURB: args.blurb,
    STACK: args.stack,
    NEW_FEATURE_PATHS: args.paths,
    LIVING_SPECS_TABLE: args.specsTable,
  };

  const agentsTmpl = fs.readFileSync(path.join(TEMPLATES, "AGENTS.md.tmpl"), "utf8");
  const claudeTmpl = fs.readFileSync(path.join(TEMPLATES, "CLAUDE.md.tmpl"), "utf8");
  const mdcTmpl = fs.readFileSync(path.join(TEMPLATES, "openspec.mdc.tmpl"), "utf8");

  const results = [];
  results.push(
    writeFile(path.join(args.root, "AGENTS.md"), render(agentsTmpl, vars), args.force)
  );
  results.push(
    writeFile(path.join(args.root, "CLAUDE.md"), render(claudeTmpl, vars), args.force)
  );
  results.push(ensureGitignoreCursor(args.root));

  if (args.cursorRule) {
    results.push(
      writeFile(
        path.join(args.root, ".cursor", "rules", "openspec.mdc"),
        render(mdcTmpl, vars),
        true // local file: always refresh
      )
    );
  }

  for (const line of results) console.log(line);
  console.log("Done. App-level rules: AGENTS.md (commit). Do not commit .cursor/.");
}

main();
