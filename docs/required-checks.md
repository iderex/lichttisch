# The checks proposed as merge conditions

Issue: #109

A required status check is matched by its literal name. Nothing infers it from
a workflow file, a job, or anybody's intention, so the set of names is a fact
that has to be written down once and read from one place afterwards. This
document is that place.

It changes no repository setting. It is the proposal; applying it to the
protection rule is the maintainer's action, taken outside the tree, and until
that action is taken every name below is a proposal and not a condition.

This document names checks, which is the one place `CONTRIBUTING.md`'s rule
against listing what a command could print does not apply. The rule exists
because a list drifts against the thing it describes and nothing notices. Here
the literal name is itself the artefact: the protection rule holds a string, and
a document that only named a command would leave that string chosen in a
settings dialogue with no reasoning attached to it. What the section below adds
instead is the commands that make a drift visible.

## What the branch requires today

    gh api repos/iderex/lichttisch/rulesets --jq '.[] | "\(.id) \(.name)"'
    20487045 gate
    gh api repos/iderex/lichttisch/rulesets/20487045 --jq '{enforcement, bypass: .bypass_actors, rules: [.rules[].type], required: [.rules[] | select(.type=="required_status_checks") | .parameters.required_status_checks[].context]}'
    {"bypass":[],"enforcement":"active","required":[],"rules":["deletion","non_fast_forward","pull_request"]}

Run on 2026-08-06. `required` is empty, so today no check is a merge condition
here. A pull request can be merged with every gate red. That is the gap this
document is written against, and it is stated plainly rather than left to be
discovered by somebody who assumed the green ticks meant something.

## What the workflows declare

The check-run name is the job's `name:` where one is set, and the job id where
none is. Several jobs in this tree deliberately set no name, each saying so in
its own file, because a job id cannot drift apart from a label somebody typed
twice. This command reads both out of the tree:

    git ls-files .github/workflows | while read -r f; do sed -n '/^jobs:/,$p' "$f" | sed -n -e "s|^  \([A-Za-z0-9_-]*\):\$|$f\tjob id: \1|p" -e "s|^    name: |$f\tjob name: |p"; done

## What a commit actually reported

The tree says what is declared. Only a commit says what reported, and the two
differ in ways worth seeing:

    gh api "repos/iderex/lichttisch/commits/$(git rev-parse origin/main)/check-runs" --jq '.check_runs[] | "\(.name)\tapp=\(.app.slug)\t\(.conclusion)"' | sort -u

    gh api repos/iderex/lichttisch/commits/2ffa9cf5a79097fb1e7bef83d4c7cbff52367de4/check-runs --jq '.check_runs[] | "\(.name)\tapp=\(.app.slug)\t\(.conclusion)"' | sort

The first reads the head of the default branch, which is a push. The second
reads the head commit of a merged pull request, which is where a merge condition
would be evaluated. Both were run on 2026-08-06 and neither returns the set the
other does.

Three differences came out of that comparison, and each one changes what may be
proposed below.

The pull-request head carries `DCO sign-off` and `dependency-review`; the
default-branch head does not. Both workflows trigger on `pull_request` only, so
this is the expected shape rather than a fault.

The default-branch head carries `Scorecard analysis`; the pull-request head does
not. `.github/workflows/scorecard.yml` declares `branch_protection_rule`, `schedule`
and `push` to `main`, and no `pull_request` trigger at all, with the reason
written in the file. So no check run of that name exists on a pull request to be required.

The pull-request head carries a check named `zizmor` from the app
`github-advanced-security`, alongside the one named `Audit workflows (zizmor)`
from `github-actions`. The first is the code-scanning result created from the
SARIF category the workflow uploads under, not the gate job. The gate job is the
second name. This matters more than it looks: the upload step that produces the
first is declared `continue-on-error` and is skipped entirely on fork and
Dependabot pull requests, so requiring `zizmor` would require an artefact the
workflow is written to survive the loss of, while leaving the audit that
actually fails the build unrequired.

Several names appear twice on the pull-request head, from two runs of one
workflow. Those workflows trigger on both `push` to every branch and
`pull_request` to every branch, and a branch pushed to this repository and then
opened as a pull request fires both. Requiring such a name requires every run
reporting it, which is the safe direction rather than the dangerous one. The
duplication itself is work for #18, where the gate set is brought under stable
names, and it is named here so that the choice below is not mistaken for an
oversight.

## The proposed merge conditions

Each line is the literal check-run name, the workflow file and job that produce
it, and what a merge without it would let through.

`build`, from `.github/workflows/build.yml`, job `build`. Prevents a merge of a
tree that does not compile, or a binary that will not start on a machine with no
display and no elevated rights.

`tests`, from `.github/workflows/tests.yml`, job `tests`. Prevents a merge that
breaks the suite or drops line coverage below the declared floor.

`format-and-lint`, from `.github/workflows/format-and-lint.yml`, job
`format-and-lint`. Prevents a merge whose shape is somebody's editor rather than
`rustfmt.toml`, and a merge tripping a lint the workspace manifest declares as
an error.

`DCO sign-off`, from `.github/workflows/dco.yml`, job `dco`. Prevents a merge
carrying a commit whose author has not asserted the certificate in `DCO.md`.

`dependency-review`, from `.github/workflows/dependency-review.yml`, job
`dependency-review`. Prevents a merge introducing or upgrading a dependency that
carries a known advisory at low severity or above.

`Reject stored carriage returns`, from `.github/workflows/line-endings.yml`, job
`carriage-return`. Prevents a merge storing a carriage return in a tracked text
blob, in a tree whose tests exist to assert on exact bytes.

`Reject Trojan Source Unicode`, from `.github/workflows/unicode-guard.yml`, job
`bidi`. Prevents a merge of source that renders to a reviewer differently from
how it runs.

`Documentation lint`, from `.github/workflows/docs-lint.yml`, job `docs-lint`.
Prevents a merge of documentation naming a path this tree does not have, a
command whose program this tree never says it has, or a decision record with no
condition that would reverse it. It judges the shape of what a document says
and never whether it is true, which is at the top of
`tools/docs-lint/src/main.rs` and is the part worth reading before treating a
green tick here as a review.

`Audit workflows (zizmor)`, from `.github/workflows/zizmor.yml`, job `zizmor`.
Prevents a merge of a workflow change reintroducing template injection, an
unpinned action, or a token wider than the job it is granted to. The name is the
job's `name:` and not the SARIF category, for the reason set out above.

## What is deliberately advisory

`Scorecard analysis` is not proposed. It produces no check run on a pull request
head, measured above, because its workflow has no `pull_request` trigger and the
file says why. A required check that never reports is not a check that passes,
and a pull request waiting for a report that cannot arrive is a branch nobody
can merge. That last sentence is a claim about how the platform treats an
unreported required check rather than something measured here; what is measured
here is the absence of the check run. Making Scorecard a merge condition would
first mean giving it a pull-request trigger, which its own file argues against,
so the honest position is that it stays a self-audit that a person reads.

`zizmor`, the code-scanning check, is not proposed, for the reason given above.
The gate it is mistaken for is proposed under its own name.

There is no third category here. Every other check this tree produces on a pull
request is proposed as a merge condition, because a gate nobody has to pass is a
gate that is off, and this project has no check it is willing to run and ignore.

## Renaming a check

A rename is a change to this document first, then to the workflow, then to the
protection rule. Any other order breaks the rule silently: the protection rule
holds the old string, the workflow reports the new one, and the check that was
required is simply absent from every pull request afterwards. Absent reads as
pending rather than as failing, so nothing goes red and nothing gets merged
either, until somebody removes the requirement to unblock the queue.

That is also why the job id is the check name wherever a job sets no `name:`.
It removes one of the two strings that can drift.

## Who applies this

The maintainer, in the repository settings, outside the tree. Nothing in this
repository can apply a protection rule to itself, and nothing here should be
read as having done so. The two commands under `## What the branch requires
today` are how a reader confirms which of the names above are conditions on the
day they are reading, rather than trusting this list to be current.
