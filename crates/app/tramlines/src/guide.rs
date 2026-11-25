//! The guide that `tramlines new` prints on stderr beside the skeleton: each
//! part of the new document, in the order of the schema, with whether it
//! is optional, the words of its guidance, and its example, so that the
//! writer reads each part's guidance and example before writing it. An
//! example that the skeleton already holds whole, such as the line of the
//! id, is left out, because the writer has it in front of them, and so is
//! the example of a section whose heading the skeleton holds, because the
//! examples of its blocks follow it, unless the section repeats.

use std::fmt::Write as _;

use tramlines_md::derive::{BlockGuide, Guide};

/// The guide as text, for the skeleton `document`: one entry for each part,
/// with its guidance and its example indented under it.
// sokf-implements: CLI-12_AC-14
pub fn text(guide: &Guide, document: &str) -> String {
    let mut g = Writer {
        out: String::from(
            "\nThe guide of the new document: each part, its guidance, and its example.\n",
        ),
        document,
    };
    let opening: Vec<&str> = guide
        .guidance
        .iter()
        .chain(&guide.bans)
        .map(String::as_str)
        .filter(|t| !t.trim().is_empty())
        .collect();
    if !opening.is_empty() {
        g.entry(0, "The document", false, Some(&opening.join(" ")), None);
    }
    for key in &guide.frontmatter {
        let name = format!("Frontmatter `{}`", key.key);
        let example = key.example.as_deref().filter(|e| !g.holds(e));
        g.entry(0, &name, key.optional, key.text.as_deref(), example);
    }
    if let Some(p) = &guide.preamble
        && (p.text.is_some() || p.blocks.iter().any(|b| g.says(b)))
    {
        g.entry(0, "The preamble", false, p.text.as_deref(), None);
        g.blocks(1, &p.blocks);
    }
    for s in &guide.sections {
        let name = format!(
            "Section `{} {}`",
            "#".repeat(usize::from(s.level)),
            s.heading
        );
        // A section that repeats gives occurrences that no block's example
        // holds, so its example stays.
        let heading = |e: &&str| e.lines().next().is_some_and(|h| g.holds(h));
        let example = s.example.as_deref().filter(|e| s.repeat || !heading(e));
        g.entry(0, &name, s.optional, s.text.as_deref(), example);
        g.blocks(1, &s.blocks);
    }
    g.out
}

/// The text of the guide so far, and the skeleton that it guides.
struct Writer<'a> {
    out: String,
    document: &'a str,
}

impl Writer<'_> {
    fn blocks(&mut self, depth: usize, list: &[BlockGuide]) {
        let said: Vec<&BlockGuide> = list.iter().filter(|b| self.says(b)).collect();
        for b in said {
            let name = format!("Block {} ({})", b.pos, b.kind.name());
            let example = b.example.as_deref().filter(|e| !self.holds(e));
            self.entry(depth, &name, b.optional, b.text.as_deref(), example);
            self.blocks(depth + 1, &b.blocks);
        }
    }

    /// Whether the guide has anything to say of `b`: its guidance, an
    /// example that the skeleton lacks, or a block inside it that it says
    /// something of.
    fn says(&self, b: &BlockGuide) -> bool {
        b.text.is_some()
            || b.example.as_deref().is_some_and(|e| !self.holds(e))
            || b.blocks.iter().any(|inner| self.says(inner))
    }

    /// Whether the skeleton holds `text` already, whole lines and all of
    /// them, in a run, each but for its indent, as a block of an item is
    /// written under the item.
    fn holds(&self, text: &str) -> bool {
        let want: Vec<&str> = text.lines().map(str::trim_start).collect();
        let have: Vec<&str> = self.document.lines().map(str::trim_start).collect();
        !want.is_empty() && have.windows(want.len()).any(|w| w == want.as_slice())
    }

    /// One part: its name, then its guidance and its example, each indented.
    fn entry(
        &mut self,
        depth: usize,
        name: &str,
        optional: bool,
        guidance: Option<&str>,
        example: Option<&str>,
    ) {
        // A key that the skeleton holds, with nothing more to say, is left
        // out.
        if depth == 0 && name.starts_with("Frontmatter") && guidance.is_none() && example.is_none()
        {
            return;
        }
        let pad = "  ".repeat(depth);
        let _ = writeln!(
            self.out,
            "\n{pad}{name}{}",
            if optional { ", optional:" } else { ":" }
        );
        if let Some(g) = guidance {
            let _ = writeln!(self.out, "{pad}  {g}");
        }
        let Some(e) = example else {
            return;
        };
        let _ = writeln!(self.out, "{pad}  Example:");
        for line in e.lines() {
            if line.is_empty() {
                self.out.push('\n');
            } else {
                let _ = writeln!(self.out, "{pad}    {line}");
            }
        }
    }
}
