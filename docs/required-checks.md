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

Several names appeared twice on the pull-request head, from two runs of one
workflow. Those workflows triggered on both `push` to every branch and
`pull_request` to every branch, and a branch pushed to this repository and then
opened as a pull request fires both. #18 closed that, and the next section is
what it settled.

## When each gate runs, and why it is not every push

Every gate here takes two triggers and no others. A pull request against any
base branch, which is the event a merge condition is evaluated on, and a push to
`main`, which is what leaves a verdict on the default branch head after a merge.

    git ls-files .github/workflows | while read -r f; do printf '%s\t' "$f"; sed -n '/^on:/,/^permissions:/p' "$f" | grep -E '^  (push|pull_request|schedule|branch_protection_rule):|^    branches:' | tr '\n' ' '; echo; done

A push to any other branch runs nothing. That is deliberate and it is a trade
rather than a tidy-up.

What it buys is one run per gate per head. A branch pushed here and then opened
as a pull request used to fire both triggers, so one head carried two check runs
under one name. A protection rule matching that name then waits on both, and a
reader counting green ticks is reading one gate twice.

What it gives up is feedback on a branch with no pull request open against it.
The only route into `main` is a pull request, because the ruleset quoted at the
top of this document refuses a direct push and has no bypass actors, so nothing
reaches the default branch without every gate having run on it. A branch nobody
has opened a pull request for is not on that route, and the cost is that its
author learns about a red gate when they open one rather than when they push.

The narrower case is a branch that is not `main` and is not protected. A pull
request against it runs every gate, because the `pull_request` trigger takes
every base branch. A push straight into it runs none. `unicode-guard.yml`
carried the opposite choice for exactly that reason and its own comment now
records what changed.

`Scorecard analysis` keeps its own shape, in its own file, for the reason that
file gives. `DCO sign-off` and `dependency-review` take `pull_request` alone,
because neither has anything to say about a push: one reads a base to head
commit range and the other reads the dependency change a pull request proposes.

Every workflow here also cancels its own superseded run:

    git ls-files .github/workflows | xargs grep -c 'cancel-in-progress: true'

A run cancelled halfway leaves nothing half done in any of them. Eight write
only inside their own runner. The two that write anywhere else upload a SARIF
file to code scanning as their last step, and losing that upload leaves the
previous analysis standing rather than a partial one, which is why neither is
the exception the condition allows for. If a gate ever gains a step that
publishes something a reader would act on, that gate is where the exception
belongs and this paragraph is what it argues against.

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

`static-analysis`, from `.github/workflows/static-analysis.yml`, job
`static-analysis`, added by #16. Prevents a merge pulling in a dependency the
advisory database has an entry against, one nobody maintains any more, one whose
licence is not on the allow list in `deny.toml`, or one arriving from a source
that file does not name. `docs/static-analysis.md` says what it is proven to
refuse and how small the graph it judges is today.

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

`Pull request hygiene`, from `.github/workflows/pr-hygiene.yml`, job
`pr-hygiene`. Prevents a merge whose body names no issue, names an issue this
repository does not have, names an issue already closed, or drops a section
`.github/pull_request_template.md` declares. It judges that the sentences exist
and not what they say, which `tools/pr-hygiene/src/main.rs` states in its own
words, and it does not judge a body written by a bot at all.

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
