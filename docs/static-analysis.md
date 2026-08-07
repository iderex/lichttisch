# The static analysis gate

Issue: #16

The formatting and lint gate judges the code written here. This one judges what
the tree pulls in, which is a different question with a different failure: an
advisory against a dependency, a crate nobody maintains, a licence nobody
agreed to, and a package arriving from a source that was never named.

It matters more here than in an average project. This tree will link image and
metadata parsers, which is where advisories in this field actually land, and a
parser reads bytes that arrived from a card, a camera or a stranger.

## The command

Read it out of the workflow that owns it, for the reason `CONTRIBUTING.md`
gives about every other gate. A second copy of a gate's command is a second
place that goes stale.

    sed -n '/run: |/,$p' .github/workflows/static-analysis.yml

The analyser is not part of the toolchain and has to be installed first. The
workflow names the exact version rather than the newest, so a verdict belongs
to one tool rather than to the day it ran.

## Where the thresholds are

`deny.toml`, at the root, and nowhere else. Not in a workflow argument: a
threshold passed on a command line is a rule a local run cannot see, and the
two stop agreeing on the day one of them changes. That file carries its
reasoning inline, one comment per setting.

There is no severity below which an advisory is tolerated here. That is worth
one sentence more than it looks, because it is not a strict value this project
chose over a lax one. The key that would set it no longer exists in the version
the workflow installs:

    cargo deny --version
    cargo-deny 0.18.6
    printf '[advisories]\nseverity-threshold = "none"\n' > /tmp/sev.toml
    cargo deny check --config /tmp/sev.toml advisories
    error[deprecated]: this key has been removed, see
    https://github.com/EmbarkStudios/cargo-deny/pull/611 for migration information

So every advisory the database carries is a finding, and a tree wanting to
tolerate one edits the `ignore` list in `deny.toml` with a reason on the line
above it, where a reader will meet it.

## What the gate is proven to refuse

A gate nobody has watched go red is a gate nobody knows is on. The last step of
the workflow builds throwaway packages and runs the real `deny.toml` against
them. Five legs, each an introduced finding paired with the one-change
neighbour that does not carry it, so a refusal cannot be read as the fixture
being broken some other way:

- a package under a licence the allow list does not carry is refused,
- the same package under a listed licence passes,
- a package pulling in a crate the advisory database has entries for is
  refused,
- the same package without that dependency passes,
- an advisories run whose database cannot be read fails rather than returning a
  clean verdict from nothing.

The last leg is the one worth reading twice. A check that reports "advisories
ok" because it had no advisories to consult is worse than no check, because it
produces the green tick somebody trusts.

Run on this repository's own tree, the whole set is:

    cargo deny check
    advisories ok, bans ok, licenses ok, sources ok

## What it does not cover

The graph it judges today is almost empty. Every entry in the lock file is a
member of this workspace and there is no third-party code in the tree at all:

    grep -c '^\[\[package\]\]' Cargo.lock
    9
    grep -c 'source = ' Cargo.lock
    0

So a green run today says almost nothing about this project's dependencies,
because it has none. What it does say is that the rules are in place and bite,
which is the point of adding the gate before the dependencies rather than
after: the first parser to arrive meets a licence allow list and an advisory
database that were already there, instead of being followed by them.

A verdict is also a statement about the advisory database on the day it ran. A
green run does not stay green, and it is not meant to: a branch that passed
last week fails today when an advisory lands against something it depends on,
and that is the gate working rather than flapping.

The wildcard rule is a warning rather than a refusal, and `deny.toml` carries
the reason at the setting. It is the one place in that file where a finding is
reported and not refused.

Nothing here judges the code in this repository. That is the formatting and
lint gate, and the two are deliberately separate: one asks whether what was
written is sound, the other asks what was brought in.
