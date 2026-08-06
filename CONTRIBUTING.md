# Contributing

## Every change starts as an issue

An issue says what is wrong, what the evidence is, and what done means. Where
the evidence is a number, it carries the command that produced it. The issue
templates ask for exactly those three things, so filling one in is the whole
obligation.

A change lands as a pull request against `main`. Direct pushes are refused by a
ruleset that has no bypass actors, so this holds for the maintainer too:

    gh api repos/iderex/lichttisch/rulesets --jq '.[] | select(.name == "gate") | .id'
    20487045
    gh api repos/iderex/lichttisch/rulesets/20487045 --jq '{enforcement, bypass: .bypass_actors, rules: [.rules[].type]}'
    {"bypass":[],"enforcement":"active","rules":["deletion","non_fast_forward","pull_request"]}

The pull request body says which issue it closes and what failure the change
prevents.

## Signing off

Every commit carries a `Signed-off-by:` trailer naming its author. Write it with
`git commit -s`. What the trailer asserts is in [DCO.md](DCO.md), and the gate
compares the trailer against the commit author, so a mismatched name or address
fails the same way a missing trailer does.

A branch written without it is repaired by rewriting the trailers onto the
existing commits:

    git rebase --signoff origin/main

Not by adding a commit that signs off on behalf of the earlier ones. The gate
reads every non-merge commit in the range, so the earlier ones are still there.

## The gates, and running them before you push

The tree is the authority for which gates exist. This document names the
command that prints them rather than listing them, because a list here would
drift against the directory it describes and nothing would notice:

    git ls-files .github/workflows

Each gate carries its own script inside its own workflow file, and that file is
the authority for what the gate actually runs. To run one locally, read the
command out of the workflow that owns it:

    sed -n '/run: |/,$p' .github/workflows/<name>.yml

This document deliberately does not copy those commands into itself. A copy
would be a second place holding a gate's logic, and the two would stop agreeing
on the day one of them changed.

There is no single command that runs every gate locally, and there is no local
hook that runs them for you. Two of the gates cannot run off GitHub at all: one
asks GitHub's advisory database about the dependencies a pull request changes,
and one is scored against the repository as the hosting platform sees it.
Neither has a local form to name. The rest are a scan of the tracked tree or of
the workflow files, and each runs anywhere its own tool is installed.

So a red gate is discovered after the push rather than before it, which is a
real cost and is written here rather than hidden. Issue #20 asked for a single
local command that runs everything; it stays open, carrying the reason there is
no such command.

## Commit messages

State what changed and what failure it prevents. Where the change is a
correction, say what was wrong and how it was found.

One topic per commit and per pull request. A commit carrying two unrelated
changes has a message describing one of them.

No tool names, no generated-by markers and no attribution to anything but a
person, in any tracked file or any commit message.

## What is enforced, and what is asked for

The difference matters more here than the rules do, because a contributor who
cannot tell them apart is misled about which failures are real.

Enforced means a check refuses the violation and the pull request goes red. The
sign-off trailer is enforced. So is the absence of a stored carriage return in a
tracked text file, and the absence of the invisible Unicode that makes source
render differently from how it runs. So is a dependency with a known advisory,
and the workflow files' own security posture. That paragraph illustrates the
kind of thing that is enforced and is not the set of them: the command above
prints the set, and a second copy of it here would drift.

Asked for means this document says it and nothing refuses it. Everything under
`## Commit messages` is asked for. So is the one-topic rule, the rule that an
issue states its evidence, and the rule that a number carries its command.
Nothing in this tree reads a commit message, an issue body or a pull request
body, so all of it is held by whoever reviews the change.

Where a rule that ought to be enforced is only asked for, that is an issue
rather than a paragraph. Say so in the issue rather than tightening the wording
here, because a firmer sentence in a document refuses nothing.

## Dependency updates

`.github/dependabot.yml` is the sending end. Routine version updates are grouped
into one pull request a week rather than a dozen. A security update is
deliberately not grouped, so it arrives on its own and is distinguishable from a
routine bump without opening it.

The two are handled differently. A routine update waits for the ordinary review
and can sit until somebody has time for it. A security update is read first,
because the receiving gate fails a pull request on a known advisory at low
severity or above, and every branch opened after the advisory lands inherits the
red. Neither is merged on the strength of being automated.

Whoever reviews any other change reviews these. What they check, in this order.
What actually moved, read from the diff rather than from the title, because the
title is generated and the diff is the change. Whether a pinned action's new sha
belongs to the tag the title claims, since a pin is only worth what the sha
behind it is. Whether the release notes describe a behaviour change rather than
a version number. And for a lock file, the lines that changed, one by one: a
lock file is the one artefact where the diff is the whole of the change, and it
is the one people are most tempted to accept unread. A lock file that is trusted
rather than read is a supply chain with no review in it.

The sign-off gate exempts two automation identities, `dependabot[bot]` and
`github-actions[bot]`, in both the plain and the numeric-prefixed spelling of
their addresses:

    git grep -c 'bot\]@users.noreply.github.com' -- .github/workflows/dco.yml
    .github/workflows/dco.yml:4

They are exempt because a bot cannot assert the certificate in [DCO.md](DCO.md)
on its own behalf; the assertion is a statement by a person about work they have
the right to submit. The allowlist names those two identities exactly rather
than matching any address shaped like a bot, so a person cannot exempt
themselves by choosing an address that ends the right way. Adding a third
identity is a change to that gate, argued in its own issue.

## Text and bytes

`.gitattributes` decides what reaches git, and
[docs/text-and-line-endings.md](docs/text-and-line-endings.md) says what is
declared and what the guard catches. Read it before adding a file whose exact
bytes matter, which in this project is any fixture.

## Where the decisions are

`docs/decisions/` holds one record per decision, numbered, added rather than
edited. A record that is overtaken is superseded by a new one that names it.
Every record answers, for its own artefact, whether the means can carry a
property a machine can refuse and a proof that runs, whether anything outside
this repository forces it, what it adds to the tree, and whether it is testable
by the suite that exists rather than by a parallel one.

Before building an artefact, answer those four questions for it. A sentence in
the issue or the pull request body naming the means and the reason it fits is
the whole obligation, and it is asked for rather than enforced.
