// Builds the source of the documentation site for mdBook, in target/book-src:
// the README as the introduction, every page of docs/ but the schemas and
// the indexes, the two specifications, and the JSON Schema of a schema.
// Each page loses its frontmatter, each sokf link becomes a link between
// pages of the book, and a link to any other file of the repository goes to
// GitHub at DOCS_REF (default `main`). SUMMARY.md lists the pages by kind,
// each kind by `order`.

import { copyFileSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, posix, relative } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const out = join(root, "target", "book-src");
const ref = process.env.DOCS_REF || "main";
const github = `https://github.com/six5536/tramlines/blob/${ref}`;

/** Every markdown file under `dir`, as a path from the root, `/`-separated. */
function walk(dir) {
  const found = [];
  for (const name of readdirSync(join(root, dir)).sort()) {
    const path = posix.join(dir, name);
    if (statSync(join(root, path)).isDirectory()) {
      found.push(...walk(path));
    } else if (name.endsWith(".md")) {
      found.push(path);
    }
  }
  return found;
}

/** The frontmatter of `text` as a map of its top-level scalars, and the body. */
function split(text) {
  if (!text.startsWith("---\n")) {
    return { front: {}, body: text };
  }
  const end = text.indexOf("\n---\n", 4);
  const front = {};
  for (const line of text.slice(4, end).split("\n")) {
    const m = line.match(/^([a-z-]+):\s*(.*)$/);
    if (m) {
      front[m[1]] = m[2].replace(/^(['"])(.*)\1$/, "$2");
    }
  }
  return { front, body: text.slice(end + 5).replace(/^\n+/, "") };
}

// The pages: their source, their path in the book, and their frontmatter.
const pages = [];
pages.push({ source: "README.md", book: "index.md", front: { title: "Introduction" } });
for (const source of walk("docs")) {
  if (source.startsWith("docs/schemas/") || source.endsWith("/index.md")) {
    continue;
  }
  const { front } = split(readFileSync(join(root, source), "utf8"));
  pages.push({ source, book: source.slice("docs/".length), front });
}
for (const name of ["tramlines-spec.md", "tramlines-md-spec.md"]) {
  const source = `knowledge/spec/${name}`;
  const { front } = split(readFileSync(join(root, source), "utf8"));
  pages.push({ source, book: `spec/${name}`, front });
}
const bookOf = new Map(pages.map((p) => [`/${p.source}`, p.book]));

/** The target of a link from the page at `from`, given as a repository path. */
function target(from, path) {
  const [file, fragment] = path.split("#");
  const book = bookOf.get(file);
  const tail = fragment === undefined ? "" : `#${fragment}`;
  if (book !== undefined) {
    const rel = posix.relative(posix.dirname(from), book) || posix.basename(book);
    return `${rel}${tail}`;
  }
  if (file.startsWith("/docs/") || file === "/README.md") {
    broken.push(`${from}: ${path}`);
  }
  return `${github}${file}${tail}`;
}

// The links to a page of the book that isn't there.
const broken = [];

/** The body of a page with its links resolved for the book. */
function rewrite(page) {
  const text = readFileSync(join(root, page.source), "utf8");
  let { body } = split(text);
  // Link definitions, as sokf writes them: `[label]: /path#fragment`.
  body = body.replace(/^(\[[^\]]+\]): (\/[^\s]*)$/gm, (_, label, path) => `${label}: ${target(page.book, path)}`);
  // A sokf link with no definition resolved to no document.
  const defined = new Set([...body.matchAll(/^\[([^\]]+)\]: /gm)].map((m) => m[1]));
  for (const m of body.matchAll(/\]\[(sokf:[^\]]+)\]/g)) {
    if (!defined.has(m[1])) {
      broken.push(`${page.book}: ${m[1]}`);
    }
  }
  // Inline links and images to a path of the repository.
  body = body.replace(/\]\((\/[^)\s]+)\)/g, (_, path) => `](${target(page.book, path)})`);
  if (page.source === "README.md") {
    // The README links files beside it; on the site they live on GitHub.
    body = body.replace(/\]\((?!https?:|#)([^)\s]+)\)/g, (_, path) => `](${github}/${path})`);
    // The README's first line is its name and its second a badge.
    body = body.replace(/^\[!\[[^\n]*\n/m, "");
  }
  return body;
}

rmSync(out, { recursive: true, force: true });
for (const page of pages) {
  const path = join(out, page.book);
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, rewrite(page));
}

// SUMMARY.md: the pages of each kind, by `order`; the error pages under
// the overview of the errors.
const byOrder = (a, b) => Number(a.front.order) - Number(b.front.order);
const of = (type) => pages.filter((p) => p.front.type === type).sort(byOrder);
const entry = (p, indent = "") => `${indent}- [${p.front.title}](${p.book})`;
const lines = ["# Summary", "", "[Introduction](index.md)", ""];
const part = (title, list) => {
  if (list.length > 0) {
    lines.push(`# ${title}`, "", ...list, "");
  }
};
part("Tutorials", of("Tutorial").map((p) => entry(p)));
part("How-to guides", of("HowTo").map((p) => entry(p)));
const reference = [];
for (const p of of("Reference")) {
  reference.push(entry(p));
  if (p.front.id === "reference-errors") {
    reference.push(...of("ErrorReference").map((e) => entry(e, "  ")));
  }
}
part("Reference", reference);
part("Explanations", of("Explanation").map((p) => entry(p)));
part("Specification", [
  "- [Core specification](spec/tramlines-spec.md)",
  "- [Markdown specification](spec/tramlines-md-spec.md)",
]);
writeFileSync(join(out, "SUMMARY.md"), lines.join("\n"));
// The JSON Schema of a schema, which the site serves beside the pages so
// that an editor can read it at a stable address.
// sokf-implements: CLI-15_AC-4
copyFileSync(join(root, "schemas", "tramlines-md.schema.json"), join(out, "tramlines-md.schema.json"));
if (broken.length > 0) {
  console.error(`docs-build: links to no page of the book:\n  ${broken.join("\n  ")}`);
  process.exit(1);
}
console.log(`docs-build: ${pages.length} pages into ${relative(root, out)}`);
