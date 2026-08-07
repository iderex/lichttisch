# Producing a number the same way every time

Issue: #25

The performance bar is a set of numbers. Numbers produced by different methods
are not comparable, so a measurement quoted in an issue, a measurement in a
decision record and a measurement in an automated check have to come out of one
harness or they are three different things wearing one name.

## The command

    cargo run --locked --release -p bench

It prints the conditions, then one row per case with the sample count, the
median and the p95. Options, which the harness also prints when it refuses an
argument:

    --samples <n>     how many times each case is timed
    --seed <n>        the seed every generated corpus comes from
    --write <path>    write the result file a later run can compare against
    --compare <path>  read a result file and print what moved

## Every number carries its conditions

    conditions
        arch=x86_64
        cpus=32
        host_identity=not recorded
        os=windows
        profile=release
        samples_asked_for=15
        seed=7811884328932370803

The profile is read from the build rather than from an argument, so a debug run
cannot be filed as a release one by whoever typed the command.

`host_identity=not recorded` is there rather than absent. Two result files can
be compared without anything knowing whether they came from one machine, and
that is a limit of this harness rather than a fact about the runs. It says so
in every file it writes.

## A skipped case is reported, never omitted

<!-- docs-lint: illustrative, the report the harness writes rather than a command -->

    case                                      samples          median             p95
    capture-a-tethered-frame                        -  skipped: no camera device present
    placeholder-digest-a-megabyte                  15      122.200 us      132.600 us
    placeholder-sort-a-hundred-thousand-keys       15        1.580 ms        1.850 ms

    3 case(s), 1 skipped. This run measured less than the whole set and must not
    be read as one that measured all of it.

The tethered case is in the list precisely because it cannot run here. A case
that vanished from the table when its hardware was missing would make a partial
run look complete, which is the failure this line exists against.

The camera check is shallow and its shallowness is the disclosure: it looks for
a device node, and a device node is not a camera that answers. It can say there
is definitely none. It is never what makes a case report itself as measured.

## The two cases that measure nothing this project does

`placeholder-digest-a-megabyte` and `placeholder-sort-a-hundred-thousand-keys`
exercise the harness so its output can be read before a real case exists. The
prefix is in their names so that a number from them is never mistaken for a
performance bar. The real cases arrive with the code they measure.

## The comparison does the subtraction

    cargo run --locked --release -p bench -- --compare target/bench/first.txt

    against target/bench/first.txt

    capture-a-tethered-frame                  skipped by both runs: no camera device present
    placeholder-digest-a-megabyte             122.200 us -> 132.800 us  +8.6 percent
    placeholder-sort-a-hundred-thousand-keys  1.580 ms -> 1.398 ms  -11.5 percent

Any condition that differs between the two files is printed above that table,
so a comparison across two machines or two profiles is possible and is not
possible quietly.

A case one run measured and the other skipped is reported as not comparable
rather than as a change. A case only one of the two has is named rather than
dropped.

Those two numbers are also the honest illustration of what this harness is not.
Eleven percent between two runs of the same code on the same machine is noise,
not a regression, and nothing here separates the two. Issue #107 is where a
number becomes a red check, and that is where the question of how many samples
and how much quiet a machine needs has to be answered.

## What the numbers are

The median and the p95 are nearest rank over the samples: every number printed
is a duration some run actually produced, rather than an interpolation between
two of them. A p95 over thirty samples is the second slowest sample, which is
worth knowing before quoting one.

Each case gets one untimed pass before the timed ones, so the first sample is
not paying for work the rest will not do. The seed moves per sample, derived
from the run's seed, so a case is not measured on one lucky arrangement of
bytes and the whole set is still reproducible from the seed in the output.

## Where it runs

No display, no elevated rights and no camera. The only case that wants hardware
is the one that reports itself skipped without it.

## What this fleet does to a number

Issue: #107

`.github/workflows/bench.yml` runs the harness twice in one job on every pull
request and prints both. It refuses nothing, and this section is why: #107 asked
for a stored baseline and a margin, and what was measured says no margin
separates a regression from what this fleet does on its own.

Two questions, and they came back with opposite answers.

**Inside one job the harness is quiet.** Two runs of one tree, one after the
other on one runner, over seven jobs. The widest gap between the two was 0.1
percent and most were 0.0. The comparison is printed by the job itself:

<!-- docs-lint: illustrative, one job's output rather than a command a reader runs -->

    placeholder-digest-a-megabyte             182.657 us -> 182.657 us   0.0 percent
    placeholder-sort-a-hundred-thousand-keys  1.690 ms -> 1.693 ms  +0.1 percent

**Across jobs it is not.** `ubuntu-latest` is a label, and behind it this fleet
handed out two processors. Four jobs on one commit, read from the run's own
logs:

    gh run view 31226232005 --attempt 1 --log | grep -E 'model name|median_ns'

That attempt and attempt 4 landed on an AMD EPYC 7763 and attempts 2 and 3 on an
AMD EPYC 9V74. `placeholder-digest-a-megabyte` came back at 161643 and 161641
nanoseconds on the first processor and 141932 twice on the second, which is 13.9
percent between them, and `placeholder-sort-a-hundred-thousand-keys` was 17.4
percent apart:

    python -c "print(round((161643 - 141932) * 100 / 141932, 1))"
    13.9
    python -c "print(round((1532290 - 1304747) * 100 / 1304747, 1))"
    17.4

**And the processor is not the whole of it.** Three more jobs, on the next
commit on the same branch, which changed the harness's argument handling and
nothing either case measures:

    gh run view 31226990032 --attempt 3 --log | grep -E 'model name|median_ns'

On the AMD EPYC 7763 the digest case came back at 161643 nanoseconds again,
identical to the commit before. On the AMD EPYC 9V74 it came back at 182650 and
182657, against 141932 on the commit before:

    python -c "print(round((182650 - 141932) * 100 / 141932, 1))"
    28.7

Reproducible on both sides: two jobs at the old number, two at the new one, none
in between. So one processor was unmoved by the change and the other moved 28.7
percent on work the change did not touch.

Why is not measured here. Code layout is the ordinary explanation for a tight
loop moving that far on one microarchitecture and not another, and this document
does not assert it, because nothing here took the measurement that would show
it. What is measured is the movement.

### What that leaves

A stored baseline and a margin cannot do the job #107 describes on this fleet
with these cases. A margin above 28.7 percent refuses nothing anybody would
ship. A margin below it reddens on a change that touched nothing, which is the
check people learn to wave through.

Two things follow, and neither is decided here.

The cases are the problem before the fleet is. Both are placeholders that
measure nothing this project does, and both are tight arithmetic loops, which is
the shape most exposed to what was measured above. A case whose cost is decoding
a file or answering a query spends its time somewhere the layout of the binary
does not reach.

Whatever the cases are, comparing a stored number against a run on another
allocation has to clear that movement first. Comparing a change against its own
base, built and measured in the same job, is the shape that does not, and it is
not what #107 asks for. That issue holds the re-plan.

Until then the numbers are printed and nothing refuses them. `tools/bench/src/gate.rs`
is the judgement a gate would use, with its near-misses, reachable from a command
line and called by no workflow, and its own first paragraph says so.
