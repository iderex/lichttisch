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
