# The structural rules nothing refuses yet

Issue: #105

The decision records state rules about shape rather than about behaviour. Some
of them are now things that fail. The rest are still sentences, and this note
is where each one says so in its own words, with the reason it cannot be
refused today and what would refuse it.

A rule left out of both halves reads exactly like a rule nobody stated, which
is why this document exists at all and why it is the second half of a pair
rather than a list of everything.

## The enforced half is printed, not written here

    cargo test -p lichttisch --test architecture_rules -- --list rule::

Those test names are the enforced list. Nothing holds a second copy of it, so a
rule that is added, renamed or deleted moves the list by construction. A
paragraph here repeating them would be the copy that goes stale.

## The unenforced half

**The catalogue is never the authority for anything the photographer can also
set somewhere else.** Stated in `docs/decisions/0002-integration-shape.md` and
applied column by column in `docs/decisions/0010-catalogue-schema.md`. Nothing
refuses a violation because there is nothing to judge:
`crates/catalogue/src/lib.rs` holds no schema, no write path and no
reconciliation. What would enforce it is a test over the schema once it exists
as types, refusing a field classified as cached that has no route back to the
source it caches, and refusing a write into such a field from anywhere but that
route. #27 lands the schema and #31 lands the reconciliation, and the rule
becomes refusable when the second of them does.

**Nothing above the catalogue composes a query in the storage engine's own
language.** Stated in `docs/layout.md`. Two halves, and neither is refused
today. The dependency half becomes free once an engine is chosen, because the
engine's client would then be a package named on one line of
`crates/module-boundaries.txt` and `crates/lichttisch/tests/module_boundaries.rs`
already refuses a module reaching a package its line does not name. The other
half is a query composed as text somewhere else and handed to the catalogue to
run, which no reading of the dependency graph sees. #6 chooses the engine and
nothing is decidable before it does.

**The foreign-function surface carries no catalogue type, no session type and
no query.** Stated as a bound rather than a measurement in
`docs/decisions/0001-means.md`. Where a foreign declaration may live is
enforced. What crosses it is not, and there is no foreign code in the tree to
judge:

    git grep -nE '^[[:space:]]*(unsafe )?extern "' -- 'crates/**/*.rs' 'tools/**/*.rs' ; echo "exit=$?"
    exit=1

Exit 1 from `git grep` is the clean answer: no match. The pathspec names both
directories the workspace declares members in, which is the reach the enforced
rule has: it read `crates` alone until
`rule::every_workspace_member_sits_under_an_area_these_rules_read` landed, and a
command quoted here narrower than the rule it stands behind is a claim about
less than the reader takes it for. #40 is where the bound becomes a number
produced by a command, and that issue is where the refusal is built.

**Tethered capture is a process of its own, and the culling surface stays up
when it dies.** Stated in `docs/decisions/0004-where-tethering-sits.md`. A
process boundary is a property of how the program runs rather than of what the
tree contains, so nothing here can read it. What would enforce it is the
contract suite in #76 for the interface and a case that kills the capture
process while a session is open, which needs the harness #81 names for what it
is. #74 chooses the transport.

**The session is a headless state machine and draws nothing.** Stated in
`docs/layout.md` and owed by #63. Half of it is refused: `session` may not
reach `surface`, which is its line in `crates/module-boundaries.txt`. The other
half is not, because the guard reads which packages a module reaches and never
what a module does with them, so a session opening a window through a package
already on its own line would pass. #106 runs the suite where there is no
display, no camera and no accelerator, which is the case that would catch it in
practice rather than by construction. This entry named #72 until #72 was closed
as a duplicate of #106, and the two differ in more than a number: #72 sat in the
session milestone and #106 sits in the parity one, so the route that would catch
this arrives later in the plan than the rule it is registered against.

**This software never deletes a photograph, never moves one and never writes
into one.** Stated in `docs/decisions/0010-catalogue-schema.md` as the reason
the catalogue models no deletion. Nothing refuses it because nothing in the
tree writes anything at all yet. #71 is where it becomes a test, and the shape
that test has to have is a refusal at the one place a path is opened for
writing rather than a search for the calls somebody remembered.

**Nothing can read a value out of a signal that had no opinion.** Stated in
`docs/decisions/0012-signal-interface.md`. The compiler holds it rather than a
check: the arm of `Opinion` carrying no opinion carries no measurement, so code
reaching for one does not compile, and there is nothing for a test in this tree
to observe at run time. What would enforce it as a test is a harness that
compiles a program and expects the compilation to fail, which is a dependency
this tree does not carry and which nothing else here would use. It is registered
in this half so that a reader does not take the absence of a test for the
absence of the rule.

**No code dependency in either direction with the generative editing project.**
Stated in `docs/decisions/0007-scope-boundary.md`. The direction this tree can
refuse is refused, and it is in the enforced list above. The other direction is
a fact about another repository's manifests, which this tree does not hold and
cannot read at the commit being pushed. It is held by whoever reviews a change
there, and no mechanism here is owed one.

## What this note does not do

It does not judge whether a rule belongs in one half or the other. That is a
decision recorded where the rule is stated, and a record added above the
watermark in `crates/lichttisch/tests/architecture_rules.rs` fails unless it
says which half applies to each rule it states.
