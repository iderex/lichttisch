# The gate this project is held to, mapped onto this project

Issue: #97

The standard here is not invented here. It is the gate already standing in front
of the default branch of the public repository `iderex/jellyfin-plugin-sso`,
adapted to a desktop application that decodes files and drives hardware.

Parity is not thirteen check names copied across. Several of that gate's checks
are about a plugin binary and a plugin catalogue and have no counterpart in a
program a photographer installs. Others have a counterpart under a different
name because the language differs. And this project needs gates that one does
not, because it will link a C++ parser, take bytes from removable media and a
cable, hold an interactive frame budget, and make published claims about how
often it is wrong about somebody's photographs.

This document maps the one onto the other. It is a plan rather than a promise,
so every entry names the issue that delivers it.

## Reading the target gate rather than remembering it

    gh api repos/iderex/jellyfin-plugin-sso/rulesets --jq '.[] | "\(.id) \(.name)"'
    18802863 Protect main and 5.0
    gh api repos/iderex/jellyfin-plugin-sso/rulesets/18802863 --jq '{enforcement, bypass:.bypass_actors, required:[.rules[].parameters.required_status_checks[]?.context]}'
    {"enforcement":"active","bypass":[],"required":["build","ABI floor build","Package (JPRM) / Build package","Package (JPRM) / Generate SBOM","CodeQL","Analyze (csharp)","DCO sign-off","Deterministic PR-hygiene checks","Enforce greppable invariants","Reject Trojan Source Unicode","Audit workflows (zizmor)","prettier","dependency-review"]}

    gh api repos/iderex/jellyfin-plugin-sso/contents/.github/workflows --jq '.[].name'

Run on 2026-08-06. The third command prints what that repository runs beyond
what it requires, which is the second half of the mapping.

What this tree runs is not listed here, for the reason `CONTRIBUTING.md` gives.
The command that prints it:

    git ls-files .github/workflows

The literal check-run names, which a protection rule matches and a document
therefore has to hold, are in `docs/required-checks.md` and are not repeated
here. Two copies of one set of strings is the drift this project keeps saying it
will not create.

## The required set, entry by entry

`build`. Matched. `.github/workflows/build.yml` builds the workspace and runs
the binary once where there is no display and no elevated rights. Delivered by
#14, landed.

`ABI floor build`. Replaced by a counterpart. That entry builds against the
oldest host application version the plugin claims to work with, and this project
has no host application to be compatible with. The equivalent question is which
operating systems and which native libraries an operator's machine has to
satisfy, and it is answered where the artefact is built. Delivered by #117.

`Package (JPRM) / Build package`. Replaced by a counterpart. The packaged
artefact here is an installable program rather than a plugin bundle, and the
route that produces it is a separate decision from the one that decides what a
version number promises. Delivered by #117 and #120.

`Package (JPRM) / Generate SBOM`. Replaced by a counterpart. A bill of materials
generated from a package manager's graph would miss exactly the components this
project's obligations attach to, which are the C and C++ libraries linked into
the binary. Delivered by #111.

`CodeQL`. Replaced by a counterpart. The capability carries over; the tool does
not, because the target's is configured for a language this tree does not
contain. Delivered by #98, with the gate itself owed by #16.

`Analyze (csharp)`. Not applicable. It is the language-specific job of the same
analysis run as the entry above, and this tree has no C# in it. Whatever
language jobs this project ends up requiring fall out of #98 rather than out of
this line.

`DCO sign-off`. Matched. `.github/workflows/dco.yml` verifies every non-merge
commit against `DCO.md`, as a first-party check rather than an external app.
Delivered by #21 and #20.

`Deterministic PR-hygiene checks`. Replaced by a counterpart, under the name
`Pull request hygiene`. `.github/workflows/pr-hygiene.yml` runs
`tools/pr-hygiene`, which refuses a body that names no issue, one that names an
issue this repository does not have, one that names an issue already closed, and
one that drops a section the template declares. Delivered by #137, landed.

What carried over is the half a machine can hold. `CONTRIBUTING.md` still states
that nothing here reads a commit message or an issue body, and that is still
true: the counterpart reads a pull request body and nothing else. What it cannot
judge is whether the failure a body claims to prevent is the real one, which is a
question about meaning, and that half stays with whoever reviews the change. The
tool's own file says so at the top rather than leaving a green tick to be read as
a review.

`Enforce greppable invariants`. Replaced by a counterpart. The invariants worth
grepping for differ, because the tree differs. Delivered by #102.

`Reject Trojan Source Unicode`. Matched, under the same name.
`.github/workflows/unicode-guard.yml`. Delivered by #21.

`Audit workflows (zizmor)`. Matched, under the same name.
`.github/workflows/zizmor.yml`, at the same severity floor and the same regular
persona. Delivered by #21.

`prettier`. Replaced by a counterpart, in two halves. Source formatting is
checked by `.github/workflows/format-and-lint.yml`, which checks and never
writes, delivered by #15 and landed. Documentation formatting is not checked at
all today, and is delivered by #103.

`dependency-review`. Matched, under the same name.
`.github/workflows/dependency-review.yml`, failing closed at low severity.
Delivered by #26.

## What the target runs without requiring it

Mutation testing. Carried over. Coverage says which lines ran, not whether a
test would have noticed them changing, and this project's coverage floor is
explicitly set low enough that it cannot be read as a judgement of the suite.
Delivered by #100.

Fuzzing. Carried over and made more important, for the reason in the additions
below. Delivered by #101.

A second static analyser. Carried over, with the same argument: one analyser
finds what it was built to find. Delivered by #99.

An end-to-end login harness. Not applicable; there is no login and no server.
The shape does carry over, which is a harness that needs something the runner
does not have, is named for what it needs, and reports a skipped case as skipped
rather than omitting it. Delivered by #81 for a camera and #87 for the
neighbouring applications.

A documentation lint. Carried over and widened, because this plan puts more in
documents than that one does: decision records, measurements and the published
quality figures. Delivered by #103.

A supply-chain self-audit. Already present here as
`.github/workflows/scorecard.yml`, on the same terms it has there, which is a
checklist a person reads rather than a gate. Delivered by #21.

## What this project needs that the target does not

A fuzzing gate on the entry points that take bytes from strangers. This is the
clearest addition. The untrusted input here is a file on a card, a sidecar
written by another application over the last ten years, and a stream over a
cable, and one of the parsers behind them is a large C++ library reached across
a foreign-function boundary. Delivered by #101.

A headless conformance gate. Every claim this plan makes about being testable
depends on the suite running with no display, no camera, no accelerator and no
elevated rights, and that is the property that quietly stops being true. It is
asserted in workflow comments today and printed by two jobs; it is not refused
anywhere. Delivered by #106.

A performance bar in the checks. The bar is a set of numbers in a decision
record, and a number in a document is a claim. This project's premise is speed
at scale, so a regression belongs in a red check rather than in a later
discovery. Delivered by #107.

A carriage-return guard. The target has no equivalent. This tree will assert on
exact bytes, so a byte nobody chose reaching a blob is a defect rather than a
cosmetic problem. Delivered by #23, landed.

A module boundary test. The catalogue reaching an image library, directly or
three levels down behind a feature flag, is the structural failure this project
is most exposed to, and cargo alone does not refuse it. Delivered by #19,
landed, with the rest of the structural rules owed by #105.

A dependency pinning and lock-drift guard. Delivered by #104, with the lock half
landed under #14.

## Which of these gate a merge here

That question has one answer and it is in `docs/required-checks.md`, which names
each proposed merge condition, names what stays advisory, and states that
applying the protection rule is the maintainer's action. This document does not
restate it.

The split falls where it does for one reason. A check that runs on a pull
request and can fail on the tree's own contents is a merge condition, because a
gate nobody has to pass is a gate that is off. A check that cannot report on a
pull request, or that scores the repository as the hosting platform sees it
rather than judging the change, is advisory, because requiring it would either
block every pull request or gate the work on something the change did not touch.

Every addition above is proposed as a merge condition when it lands, except
where its own issue argues otherwise. #107 is the one that carries that argument
explicitly, because a performance threshold on a noisy runner is a check people
learn to ignore, and its issue requires the advisory or blocking choice to be
made deliberately and written down.
