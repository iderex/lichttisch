# The code scanning gate, and what it is worth

Issue: #98

The gate this project's quality is measured against carries a code scanning
analysis, and `docs/quality-parity.md` records that the capability carries over
to this tree. This note is the argued half of that entry: which findings red a
check here, why the line sits where it does, and what a green result is not
evidence of.

The authority for what runs is `.github/workflows/code-scanning.yml`. Nothing
here repeats the commands in it, for the reason `CONTRIBUTING.md` gives about a
document holding a second copy of a gate's logic. The command that reads them
out of the file:

    sed -n '/run: |/,$p' .github/workflows/code-scanning.yml

## Where the floor sits, and why

A finding carrying a security severity of 0.1 or higher fails the check. That is
the lowest score an analysis attaches to a security finding, so the practical
statement is that every security finding fails it.

The line is set there to agree with the gate next to it.
`.github/workflows/dependency-review.yml` refuses a dependency introduced or
upgraded by a pull request when it carries an advisory at low severity or above.
A tree that refuses a low-severity defect arriving as somebody else's code and
accepts the same defect written in its own would be holding its dependencies to
a standard it does not hold itself to.

What it costs is noise, and the cost is not symmetrical with the benefit. A low
severity finding is often a pattern that is wrong in general and harmless here,
and the answer to one is a change, an exclusion argued in this file, or the
floor moving. What makes the cost affordable today is the size of the tree: the
modules under `crates/` are nearly empty, so there is very little for a query to
be wrong about.

The condition that raises it: when the analysis starts refusing changes for
findings that are reviewed and judged not to be defects, and the count of those
in a month is large enough that people begin reading a red check as noise. At
that point the floor moves up by one severity band, in an issue, with the
findings that forced it quoted. It does not move because one change was
inconvenient.

## What is asked of the analysis

More than the default set, in two ways, and both are in the configuration block
of the workflow file.

The extended security suite, which adds the lower-precision security queries.
This tree is at the stage where a false positive costs a conversation and a
missed defect costs a photographer's archive.

The local threat model, which is what makes a command-line argument, an
environment variable and the contents of a file count as untrusted input.
Without it this program has almost no sources at all. It takes bytes from
removable media, from a cable and from a folder somebody points it at, and none
of those is a network request. The queries this reaches are the ones this
project's own risk is in: a path built from something the process was handed,
and the memory-safety surface that arrives with the foreign boundary once there
is one.

## What a green result is not

It is not a review. A query finds the shapes it knows, and nothing in this gate
judges whether the code does what its issue said it would.

It is not a statement about every finding the analysis produced. Only findings
carrying a security severity are compared against the floor. A finding with no
severity attached, which is what the quality queries produce, is counted and
printed by the judgement and is not what reds the check. The run prints how many
results it examined and how many carried no score, so a reader can see the
difference rather than assume it.

It is not a statement about the whole tree either, and on a pull request it is
narrower than that in a way worth naming. The analysis is source-only for this
language, so what it sees is what the workspace resolves to at that commit. On
top of that, when the run belongs to a pull request the analysis restricts the
alerts it keeps to the lines that pull request changed. It says so in its own
log, which prints `Computing PR diff ranges` and then hands a diff range
extension pack to the query run. So a green gate on a pull request says the
change introduced no finding, not that the tree holds none. The whole-tree
statement is the run on the default branch after the merge, which belongs to no
pull request and therefore has no diff to be restricted to.

That restriction is left on for the gate, because there the change is exactly
what is being judged. It is turned off in the near-miss job, and the reason is
below.

## The two near-misses

A gate nobody has watched go red is a gate nobody knows is on, and there are two
different things here that can be silently wrong.

The floor can be misread. A judgement that compares the wrong property, or that
compares with a strict inequality where it means an inclusive one, passes
exactly the finding it exists to refuse and looks identical from outside. The
gate step runs its own judgement over two documents that differ from each other
by one step in severity, one at the floor and one under it, and fails unless the
first is refused and the second is not.

The analysis can be blind. A gate wired to an analyser that reports nothing in
this language would stay green forever and would say nothing at all. The second
job writes a program that builds a path out of an argument it was handed and
opens it without a check, analyses that program the same way, and fails unless
the analysis reports it.

Neither near-miss is a file committed to this tree. Both are written when the
job runs, which is the shape the proof in
`.github/workflows/static-analysis.yml` already uses. A tracked program carrying
a deliberate defect would sit in the repository's scanning surface for as long
as it existed, and a finding nobody intends to fix is how a scanning surface
stops being read.

The second one also runs with the diff restriction turned off, and that is the
second correction rather than a preference. The program it analyses is written
by the job, so it appears in no pull request's diff, and every alert about it
was discarded before the check could read one. Both failures produced the same
symptom: an analysis that reported nothing, which reads as an analyser that
cannot see the language. Neither was that.

The second one is written into the job's own working directory, and the job
checks this repository out nowhere. That is not tidiness. The first attempt
wrote the program into the runner's temporary directory and told the analysis
to treat it as the source root, alongside an ordinary checkout, and the analysis
indexed the checkout instead: it scanned this project's own crates, found
nothing to report in them, and failed. A reader of that failure would have
concluded that the analyser is blind to this language, which was not what had
happened. What the source root moves is where results are reported from. What
decides which files are indexed is the directory the analysis runs in, so an
empty one holding only the program is the only arrangement in which this job's
verdict is about the program.

## The check names

`code-scanning` is the analysis and the judgement. `code-scanning-near-miss` is
the proof that the analysis sees the defect it is pointed at. Both take the job
id as their name, for the reason `docs/required-checks.md` gives, and that
document is where the first is proposed as a merge condition.
