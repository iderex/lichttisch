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

It is a statement about the workspace at that commit, and not about the
repository. The analysis is source-only for this language, so what it sees is
what the workspace resolves to, and a file no member pulls in is a file no query
was run over.

It is a statement about the whole of that workspace on both events, and it was
not always. Left at its default, the analysis restricts a pull request's alerts
to the lines that pull request changed: the log prints `Computing PR diff ranges`
and hands a diff range extension pack to the query run. The gate now turns that
off, in the `env` block of the job in
`.github/workflows/code-scanning.yml`, so a pull request and the push that
follows the merge ask one question rather than two. The next section is what
forced it.

## What the two events used to answer, and what that cost

The gate went red on every push to the default branch from the day it landed and
green on every pull request, and both were correct answers to different
questions. Twenty results, one query and one severity, read out of the last of
those pushes:

    gh run view 31268516008 --log-failed | grep -oE 'over  rust/path-injection security-severity=7.5 [^ ]+$' | awk '{print $NF}' | sort | uniq -c
          5 crates/catalogue/src/lock.rs
          4 crates/catalogue/tests/single_writer.rs
         11 tools/corpus/src/write.rs

None of the twenty fell on a line any pull request had changed, so the diff
restriction discarded all of them before the judgement on a pull request saw
one, and the same twenty were there for the judgement on the push afterwards.
A check that answers a narrower question before a merge than after it cannot be
a merge condition, which is what `docs/required-checks.md` proposes this one as.
Turning the restriction off is what makes the check a pull request passes the
same check the default branch runs.

What it costs is the whole workspace analysed once per pull request rather than
the lines it changed, and a pull request refused for a finding it did not
introduce. The second is the harder one and it is deliberate. A finding nobody
introduced is still a finding in the tree, and the alternative is the
arrangement this section describes, where it is nobody's and lands anyway.

## What the twenty were

One query, `rust/path-injection`, at security severity 7.5, in
`tools/corpus/src/write.rs`, `crates/catalogue/src/lock.rs` and
`crates/catalogue/tests/single_writer.rs`. Every one of the twenty flows began
at the same call in the same two fixtures: `std::env::temp_dir()`, once in the
scratch helper of each. The local threat model asked for above is what makes an
environment read untrusted input, so a scratch directory built from it was a
tainted path, and it carried the taint into the functions the fixtures called.
Five of the twenty were reported inside the locking module, which takes a
directory from its caller and never reads the environment itself.

So the answer to all twenty is one answer, and it is neither an exclusion nor a
move in the floor. The two fixtures now take their scratch root from a value the
compiler resolves rather than one the process reads: the directory Cargo sets
aside for an integration test in the first, and the workspace directory this
tool's own default output already goes to in the second. Nothing about the
program that ships changed, because nothing about the program that ships was
what the query had found.

What that does not cover is the case the finding names in production, where a
path really is assembled from something the process was handed. The corpus tool
takes an output directory on the command line and the catalogue takes one from
its caller, and neither validates it. That is untouched here and it is not the
same question: a photographer naming their own folder is not an attacker, and a
path arriving from a sidecar, a card or another program is a different input
from one an operator typed. The query is still on, at the same floor, over both
of those surfaces, so the day one of them is reached from an input this program
did not get from a person, the gate says so.

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

The second one had the diff restriction turned off before the gate did, and that
was a correction rather than a preference. The program it analyses is written by
the job, so it appears in no pull request's diff, and every alert about it was
discarded before the check could read one. Both failures produced the same
symptom: an analysis that reported nothing, which reads as an analyser that
cannot see the language. Neither was that.

Which is worth reading beside the section above. The same restriction was wrong
in both jobs, for reasons that look unrelated and are the same one: an analysis
told to keep only what a diff touched says nothing about anything the diff did
not touch, and in neither job is the diff the subject. It was corrected in one
of them first because that is where it failed loudly.

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
