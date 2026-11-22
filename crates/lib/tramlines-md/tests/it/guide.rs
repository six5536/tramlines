//! The guide that a host asks for: the guidance and the limits of each part
//! of a schema that some states admit, in the order of the schema, with
//! the words of the scaffold's guidance lines.

use serde_json::json;
use tramlines_md::{
    check::{Counts, Host, NullHost},
    derive::{derive_one, guide},
    grammar::{LimitKind, Schema, States},
};

use crate::{
    common::{compile_ok, states},
    limits::{LeadHost, hard_lead},
};

/// An issue that is open or closed: a summary, a plan while it is open,
/// and an outcome once it is closed.
const ISSUE: &str = r"line-limit: 0
guidance: An issue records one piece of work.
lifecycle:
  - key: state
    states: [open, closed]
    guidance: An issue is open while its work remains.
frontmatter:
  title:
    guidance: The title names the work.
    example: Descale the kettle
  tags:
    list: true
    optional: true
    max: { hard: 3, when-at-limit: 'Name further topics in the summary.' }
    example: kitchen
sections:
  - heading: { from: title }
    level: 1
    guidance: The opening says why the work matters.
    blocks:
      - kind: prose
        guidance: The summary says what the work changes.
        max: { soft: 1, when-at-limit: 'Put the details in the plan.' }
        example: The kettle needs descaling before it breaks.
  - heading: Plan
    level: 2
    only: { state: [open] }
    blocks:
      - kind: numbered-list
        rules:
          - pattern: '^Step: '
            rule-id: steps
            max: { hard: 5, when-at-limit: 'Split a longer plan into issues.' }
            example: 'Step: buy vinegar.'
  - heading: Outcome
    level: 2
    optional: true
    only: { state: [closed] }
    blocks:
      - kind: prose
        example: The kettle boils in half the time.
";

// The guide at a state that a document is moving to lists only the parts of
// that state, each with its limits and the words of its guidance line.
// sokf-tests: TLMD_233
// sokf-tests: MD-13_AC-1
#[test]
fn the_guide_lists_the_parts_of_the_given_states() {
    let schema = compile_ok(ISSUE);
    let open = guide(&schema, &states(&[("state", "open")]), &NullHost);
    assert_eq!(
        open.guidance.as_deref(),
        Some("An issue records one piece of work.")
    );
    let keys: Vec<(&str, bool)> = open
        .frontmatter
        .iter()
        .map(|k| (k.key.as_str(), k.lifecycle))
        .collect();
    assert_eq!(
        keys,
        vec![("title", false), ("tags", false), ("state", true)]
    );
    let tags = &open.frontmatter[1];
    assert!(tags.optional);
    let limit = tags.limit.as_ref().expect("a limit");
    assert_eq!((limit.kind, limit.count), (LimitKind::Hard, 3));
    assert_eq!(
        tags.text.as_deref(),
        Some("`hard: 3`, `when-at-limit`: Name further topics in the summary.")
    );
    let headings: Vec<&str> = open.sections.iter().map(|s| s.heading.as_str()).collect();
    assert_eq!(headings, vec!["<title>", "Plan"]);
    let summary = &open.sections[0].blocks[0];
    assert_eq!(summary.limits.len(), 1);
    assert_eq!(summary.limits[0].rule, None);
    assert_eq!(summary.limits[0].limit.kind, LimitKind::Soft);
    assert_eq!(
        summary.text.as_deref(),
        Some(
            "The summary says what the work changes. `soft: 1`, `when-at-limit`: Put the details in the plan."
        )
    );
    let plan = &open.sections[1].blocks[0];
    assert!(plan.limits[0].rule.is_some());
    assert_eq!(plan.limits[0].limit.count, 5);

    let closed = guide(&schema, &states(&[("state", "closed")]), &NullHost);
    let headings: Vec<(&str, bool)> = closed
        .sections
        .iter()
        .map(|s| (s.heading.as_str(), s.optional))
        .collect();
    assert_eq!(headings, vec![("<title>", false), ("Outcome", true)]);
}

// Each text of the guide is the words of the guidance line that the
// scaffold of the same host writes for the same part at the same states,
// and the host's words for a limit change no `Limit` of the guide.
// sokf-tests: MD-13_AC-2
// sokf-tests: MD-5_AC-15
// sokf-tests: TLMD_233
#[test]
fn the_guide_and_the_scaffold_hold_the_same_words() {
    let schema = compile_ok(ISSUE);
    same_words(&schema, &NullHost);
    let host = LeadHost(hard_lead);
    same_words(&schema, &host);
    let open = guide(&schema, &states(&[("state", "open")]), &host);
    let tags = &open.frontmatter[1];
    assert_eq!(
        tags.text.as_deref(),
        Some("At most 3 `tags`. When at the limit: Name further topics in the summary.")
    );
    assert_eq!(
        tags.limit.as_ref().map(|l| l.when_at_limit.as_str()),
        Some("Name further topics in the summary.")
    );
    // A unit rule's limit carries what it is applied to as plain data.
    let step = &open.sections[1].blocks[0].limits[0];
    assert_eq!(
        (step.format.as_deref(), step.rule_id.as_deref(), step.counts),
        (None, Some("steps"), Some(Counts::Units))
    );
    assert_eq!(
        open.sections[1].blocks[0].text.as_deref(),
        Some("At most 5 `steps` units. When at the limit: Split a longer plan into issues.")
    );
    // This host gives no words for the soft limit of the summary, which
    // then takes the limit in the syntax of the schema.
    assert_eq!(
        open.sections[0].blocks[0].text.as_deref(),
        Some(
            "The summary says what the work changes. `soft: 1`, `when-at-limit`: Put the details in the plan."
        )
    );
}

/// Check that each text of the guide that `host` gives is a guidance line
/// of the scaffold that `host` derives, at the open state.
fn same_words(schema: &Schema, host: &dyn Host) {
    let at = states(&[("state", "open")]);
    let scaffold = derive_one(schema, &at, host).expect("the schema derives");
    let open = guide(schema, &at, host);
    let mut texts: Vec<String> = open
        .frontmatter
        .iter()
        .filter_map(|k| k.text.clone())
        .collect();
    for section in &open.sections {
        texts.extend(section.text.clone());
        texts.extend(section.blocks.iter().filter_map(|b| b.text.clone()));
    }
    assert_eq!(texts.len(), 6, "{texts:?}");
    for text in texts {
        assert!(
            scaffold.contains(&format!("tramlines:guidance {text}")),
            "`{text}` is not a guidance line of\n{scaffold}"
        );
    }
}

// The guide serialises as it stands, with the grammar's names for the kind
// of a block and the kind of a limit.
// sokf-tests: MD-13_AC-3
#[test]
fn the_guide_serialises_with_the_names_of_the_grammar() {
    let schema = compile_ok(ISSUE);
    // sokf:begin guide
    // The guidance of each part of a document at its states, or at the
    // states that it is moving to, as a host shows it to the writer.
    let open = guide(&schema, &states(&[("state", "open")]), &NullHost);
    let json = serde_json::to_value(&open).expect("the guide serialises");
    // sokf:end guide
    assert_eq!(json["sections"][0]["blocks"][0]["kind"], json!("prose"));
    assert_eq!(
        json["sections"][0]["blocks"][0]["limits"][0]["limit"],
        json!({
            "kind": "soft",
            "count": 1,
            "when_at_limit": "Put the details in the plan.",
        })
    );
    assert_eq!(
        json["sections"][1]["blocks"][0]["kind"],
        json!("numbered-list")
    );
    assert_eq!(
        json["sections"][1]["blocks"][0]["limits"][0]["counts"],
        json!("units")
    );
    assert_eq!(
        json["sections"][0]["blocks"][0]["limits"][0]["counts"],
        json!(null)
    );
    // A line rule's limit counts lines.
    let fence = compile_ok(
        "line-limit: 0\nsections:\n  - heading: Note\n    level: 1\n    blocks:\n      - kind: code\n        lang: ini\n        each:\n          lines:\n            - template: '${name:slug} = ${}'\n              rule-id: setting\n              max: { hard: 3, when-at-limit: 'Put further settings in a fence of their own.' }\n              example: 'mode = eco'\n",
    );
    let json = serde_json::to_value(guide(&fence, &States::default(), &NullHost))
        .expect("the guide serialises");
    assert_eq!(
        json["sections"][0]["blocks"][0]["limits"][0]["counts"],
        json!("lines")
    );
    // A host can ask for states that admit no gated part.
    let none = guide(&schema, &States::default(), &NullHost);
    assert_eq!(none.sections.len(), 1);
}
