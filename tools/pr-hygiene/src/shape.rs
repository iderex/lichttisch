//! The judgement, as pure functions over text (#137).
//!
//! Nothing here reads a file, a network or a clock. Everything the verdict
//! depends on arrives as a string, which is what lets every rule below carry a
//! near-miss beside it: the body with the reference removed and the body with
//! it present are two fixtures rather than two pull requests.
//!
//! The shape a body has to have is read out of
//! `.github/pull_request_template.md` rather than written here. A contributor
//! who fills the template in cannot fail the check, and a maintainer who
//! changes what a body owes changes it in the file contributors actually see
//! rather than in a workflow nobody opens.

/// What the template says a body owes.
#[derive(Debug, PartialEq, Eq)]
pub struct Shape {
    /// The word the template uses in front of the issue number.
    pub keyword: String,
    /// Every second-level heading the template carries, in its own order.
    pub headings: Vec<String>,
}

/// One thing a body did not do, and the line that showed it.
#[derive(Debug, PartialEq, Eq)]
pub struct Refusal {
    /// What was missing, in the words a contributor has to act on.
    pub what: String,
    /// The line the verdict was read from, quoted rather than described.
    pub quoted: String,
}

impl Refusal {
    fn new(what: impl Into<String>, quoted: impl Into<String>) -> Self {
        Self {
            what: what.into(),
            quoted: quoted.into(),
        }
    }
}

/// The state of one issue, as the index reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Open,
    Closed,
}

/// One row of the issue index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Known {
    pub number: u64,
    pub state: State,
}

/// The issue index, as `<number> <STATE>` lines.
///
/// A row the reader cannot place is an error rather than a row to skip. An
/// index that silently lost a row would turn an issue that exists into one the
/// verdict calls unknown, which is a red check for a reason that is not true.
///
/// # Errors
///
/// Returns the offending line where a row is not two fields, where the number
/// is not a number, or where the state is neither `OPEN` nor `CLOSED`.
pub fn read_index(text: &str) -> Result<Vec<Known>, String> {
    let mut rows = Vec::new();
    for line in text.lines() {
        let row = line.trim();
        if row.is_empty() {
            continue;
        }
        let mut fields = row.split_whitespace();
        let (Some(number), Some(state), None) = (fields.next(), fields.next(), fields.next())
        else {
            return Err(format!("index row is not <number> <state>: {row}"));
        };
        let number = number
            .parse::<u64>()
            .map_err(|_| format!("index row has no issue number: {row}"))?;
        let state = match state {
            "OPEN" => State::Open,
            "CLOSED" => State::Closed,
            other => return Err(format!("index row has an unknown state {other}: {row}")),
        };
        rows.push(Known { number, state });
    }
    Ok(rows)
}

/// The shape the template declares.
///
/// # Errors
///
/// Returns a sentence naming what the template did not carry, because a
/// template that declares no shape leaves this check with nothing to require
/// and passing then would be fail-open.
pub fn shape_of(template: &str) -> Result<Shape, String> {
    let stripped = without_comments(template);
    let keyword = stripped.lines().find_map(closing_keyword).ok_or_else(|| {
        String::from("the template carries no <word> # line to read a keyword from")
    })?;
    let headings: Vec<String> = stripped
        .lines()
        .map(str::trim_end)
        .filter(|line| line.starts_with("## "))
        .map(ToOwned::to_owned)
        .collect();
    if headings.is_empty() {
        return Err(String::from("the template carries no second-level heading"));
    }
    Ok(Shape { keyword, headings })
}

/// The keyword on a line of the form `Closes #`, if the line is one.
fn closing_keyword(line: &str) -> Option<String> {
    let mut fields = line.split_whitespace();
    let word = fields.next()?;
    let reference = fields.next()?;
    if fields.next().is_some() || !reference.starts_with('#') {
        return None;
    }
    if !word.chars().all(char::is_alphabetic) {
        return None;
    }
    Some(word.to_owned())
}

/// Every issue number the body closes, with the line each was read from.
fn closing_references(body: &str, keyword: &str) -> Vec<(u64, String)> {
    let mut found = Vec::new();
    for line in body.lines() {
        let quoted = line.trim();
        let mut fields = quoted.split_whitespace();
        let (Some(word), Some(reference)) = (fields.next(), fields.next()) else {
            continue;
        };
        if !word.eq_ignore_ascii_case(keyword) {
            continue;
        }
        let Some(digits) = reference.strip_prefix('#') else {
            continue;
        };
        if let Ok(number) = digits.parse::<u64>() {
            found.push((number, quoted.to_owned()));
        }
    }
    found
}

/// The text with every HTML comment removed.
///
/// The template is mostly guidance inside comments, and a body that keeps the
/// guidance has said nothing. Removing the comments first is what makes the
/// difference between a section filled in and a section left as it arrived.
fn without_comments(text: &str) -> String {
    let mut kept = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find("<!--") {
        kept.push_str(&rest[..open]);
        let after = &rest[open + 4..];
        match after.find("-->") {
            Some(close) => rest = &after[close + 3..],
            // An unterminated comment swallows the remainder, which is what a
            // renderer does with it too.
            None => return kept,
        }
    }
    kept.push_str(rest);
    kept
}

/// Whether a body line carries the heading the template declares.
///
/// The template's words are the stem rather than the whole string, so a body
/// that writes `## What was not done, and what is bounded` has carried
/// `## What was not done`. Elaborating a heading is a thing people do and it
/// is not the failure this check is about; dropping the section is.
fn is_heading_for(line: &str, heading: &str) -> bool {
    line.starts_with(heading)
}

/// Whether the section under `heading` carries anything a person wrote.
///
/// The section ends at the next second-level heading, whether or not the
/// template declares that one, because a body that adds a section of its own
/// has still ended the previous one.
fn section_has_prose(body: &str, heading: &str) -> bool {
    let mut inside = false;
    for line in body.lines().map(str::trim_end) {
        if !inside {
            inside = is_heading_for(line, heading);
            continue;
        }
        if line.starts_with("## ") {
            return false;
        }
        if !line.trim().is_empty() {
            return true;
        }
    }
    false
}

/// Every way this body fails the shape, in the order a reader repairs them.
///
/// `index` is the issue index this repository reported. An empty index is
/// refused by the caller rather than here, because "no issue exists" and "the
/// index could not be read" are opposite statements.
#[must_use]
pub fn judge(body: &str, shape: &Shape, index: &[Known]) -> Vec<Refusal> {
    let body = without_comments(body);
    let mut refusals = Vec::new();

    let references = closing_references(&body, &shape.keyword);
    if references.is_empty() {
        refusals.push(Refusal::new(
            format!(
                "the body names no issue. Write `{} #<number>` on a line of its own, \
                 as the template does.",
                shape.keyword
            ),
            "(no line in the body carries the keyword)",
        ));
    }
    for (number, quoted) in &references {
        match index.iter().find(|known| known.number == *number) {
            None => refusals.push(Refusal::new(
                format!("issue #{number} does not exist in this repository"),
                quoted,
            )),
            Some(known) if known.state == State::Closed => refusals.push(Refusal::new(
                format!("issue #{number} is already closed, so this reference is stale"),
                quoted,
            )),
            Some(_) => {}
        }
    }

    for heading in &shape.headings {
        if !body
            .lines()
            .map(str::trim_end)
            .any(|line| is_heading_for(line, heading))
        {
            refusals.push(Refusal::new(
                format!("the body carries no `{heading}` section, which the template declares"),
                "(the heading is absent)",
            ));
        } else if !section_has_prose(&body, heading) {
            refusals.push(Refusal::new(
                format!(
                    "the `{heading}` section is empty once the template's guidance is \
                     removed, so it says nothing"
                ),
                heading,
            ));
        }
    }

    refusals
}

#[cfg(test)]
mod tests {
    use super::{Known, Refusal, State, judge, read_index, section_has_prose, shape_of};

    const ELABORATED: &str = "Closes #137

## What failure this prevents

        A body naming nothing merges.

## What was run, and what came back, in full

        cargo test, green.
";

    const TEMPLATE: &str = "<!--\nguidance nobody has to read\n-->\n\nCloses #\n\n\
        ## What failure this prevents\n\n<!-- a hint -->\n\n\
        ## What was run, and what came back\n\n<!-- another hint -->\n";

    fn shape() -> super::Shape {
        match shape_of(TEMPLATE) {
            Ok(shape) => shape,
            Err(why) => panic!("the fixture template must parse: {why}"),
        }
    }

    fn index() -> Vec<Known> {
        vec![
            Known {
                number: 137,
                state: State::Open,
            },
            Known {
                number: 97,
                state: State::Closed,
            },
        ]
    }

    fn good_body() -> String {
        String::from(
            "Closes #137\n\n## What failure this prevents\n\nA body naming nothing merges.\n\n\
             ## What was run, and what came back\n\ncargo test, green.\n",
        )
    }

    fn whats(refusals: &[Refusal]) -> Vec<String> {
        refusals.iter().map(|one| one.what.clone()).collect()
    }

    #[test]
    fn the_template_declares_the_keyword_and_its_headings() {
        let shape = shape();
        assert_eq!(shape.keyword, "Closes");
        assert_eq!(
            shape.headings,
            vec![
                String::from("## What failure this prevents"),
                String::from("## What was run, and what came back"),
            ]
        );
    }

    #[test]
    fn a_template_with_no_reference_line_is_an_error() {
        assert!(shape_of("## Only a heading\n\nprose\n").is_err());
    }

    #[test]
    fn a_template_with_no_heading_is_an_error() {
        assert!(shape_of("Closes #\n").is_err());
    }

    #[test]
    fn a_filled_in_template_passes() {
        assert_eq!(judge(&good_body(), &shape(), &index()), Vec::new());
    }

    // The near-miss the issue asks for by name: the same body, one line short.
    #[test]
    fn the_same_body_with_the_reference_removed_is_refused() {
        let near_miss = good_body().replace("Closes #137\n", "");
        let refusals = judge(&near_miss, &shape(), &index());
        assert_eq!(refusals.len(), 1);
        assert!(refusals[0].what.contains("names no issue"));
    }

    #[test]
    fn a_reference_to_an_issue_that_does_not_exist_is_refused() {
        let near_miss = good_body().replace("#137", "#13700");
        let refusals = judge(&near_miss, &shape(), &index());
        assert_eq!(
            whats(&refusals),
            vec![String::from(
                "issue #13700 does not exist in this repository"
            )]
        );
        assert_eq!(refusals[0].quoted, "Closes #13700");
    }

    #[test]
    fn a_reference_to_a_closed_issue_is_refused() {
        let near_miss = good_body().replace("#137", "#97");
        let refusals = judge(&near_miss, &shape(), &index());
        assert_eq!(
            whats(&refusals),
            vec![String::from(
                "issue #97 is already closed, so this reference is stale"
            )]
        );
    }

    #[test]
    fn a_reference_written_with_a_leading_zero_is_the_same_issue() {
        let neighbour = good_body().replace("#137", "#0137");
        assert_eq!(judge(&neighbour, &shape(), &index()), Vec::new());
    }

    #[test]
    fn a_missing_section_is_refused_and_named() {
        let near_miss = good_body().replace("## What failure this prevents\n", "");
        let refusals = judge(&near_miss, &shape(), &index());
        assert_eq!(refusals.len(), 1);
        assert!(refusals[0].what.contains("What failure this prevents"));
    }

    #[test]
    fn a_section_holding_only_the_templates_guidance_is_refused() {
        let near_miss = "Closes #137\n\n## What failure this prevents\n\n<!-- a hint -->\n\n\
             ## What was run, and what came back\n\ncargo test, green.\n";
        let refusals = judge(near_miss, &shape(), &index());
        assert_eq!(refusals.len(), 1);
        assert!(refusals[0].what.contains("is empty once"));
    }

    #[test]
    fn one_word_of_prose_is_the_neighbour_that_passes() {
        let neighbour = "Closes #137\n\n## What failure this prevents\n\n<!-- a hint -->\nnothing.\n\n\
             ## What was run, and what came back\n\ncargo test, green.\n";
        assert_eq!(judge(neighbour, &shape(), &index()), Vec::new());
    }

    #[test]
    fn the_keyword_is_matched_however_it_was_cased() {
        let neighbour = good_body().replace("Closes", "closes");
        assert_eq!(judge(&neighbour, &shape(), &index()), Vec::new());
    }

    #[test]
    fn a_reference_inside_a_sentence_is_not_read_as_a_closing_line() {
        let near_miss = good_body().replace("Closes #137\n", "This closes #137 eventually\n");
        let refusals = judge(&near_miss, &shape(), &index());
        assert_eq!(refusals.len(), 1);
        assert!(refusals[0].what.contains("names no issue"));
    }

    #[test]
    fn a_section_ending_at_the_next_heading_is_not_read_as_filled_in() {
        assert!(!section_has_prose("## One\n\n## Two\n\nprose\n", "## One"));
        assert!(section_has_prose("## One\n\nprose\n\n## Two\n", "## One"));
    }

    #[test]
    fn a_heading_the_body_elaborated_still_carries_the_section() {
        assert_eq!(judge(ELABORATED, &shape(), &index()), Vec::new());
    }

    #[test]
    fn a_heading_the_body_shortened_does_not() {
        let near_miss = ELABORATED.replace(
            "## What was run, and what came back, in full",
            "## What was run",
        );
        let refusals = judge(&near_miss, &shape(), &index());
        assert_eq!(refusals.len(), 1);
        assert!(
            refusals[0]
                .what
                .contains("What was run, and what came back")
        );
    }

    #[test]
    fn the_index_reads_what_the_repository_reported() {
        let rows = read_index("137 OPEN\n\n97 CLOSED\n");
        assert_eq!(rows, Ok(index()));
    }

    #[test]
    fn an_index_row_the_reader_cannot_place_is_an_error() {
        assert!(read_index("137\n").is_err());
        assert!(read_index("137 OPEN CLOSED\n").is_err());
        assert!(read_index("one-three-seven OPEN\n").is_err());
        assert!(read_index("137 MERGED\n").is_err());
    }
}
