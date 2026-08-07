# A hundred thousand files that can be made again

Issue: #4

The readme claims a hundred thousand-plus images. No issue on this board may
assert a number about that scale without a corpus that produced the number, so
the corpus exists before the engines are measured.

Real photographs cannot be shipped for this. They are large, and they are other
people's pictures, which is the thing this project exists not to move around.
So the set is generated from a seed rather than collected, and a second machine
reproduces it rather than downloading it.

## The command

    cargo run --locked --release -p corpus -- --files 100000 --seed 4242424242 --byte-divisor 2000 --out target/corpus

It prints the conditions, then what it produced. Every option, which the tool
also prints when it refuses an argument:

    --out <path>               where the corpus goes (default target/corpus)
    --files <n>                how many files (default 5000)
    --seed <n>                 the seed everything comes from (default 4242424242)
    --cards <n>                how many cards the frames came off (default 6)
    --years <n>                how many years the timestamps span (default 4)
    --malformed-permille <n>   damaged files per thousand (default 10)
    --byte-divisor <n>         divide every planned length by this (default 1)
    --dry-run                  say what it would cost and write nothing

An argument nobody understood is refused rather than ignored, because a run
that silently dropped `--files 100000` would report five thousand files under a
heading somebody read as a hundred thousand.

## What is realistic here, and what is not

Not the picture. Every file holds a short readable header and filler bytes from
a generator, so the corpus says nothing whatever about what is in a photograph.

What is realistic is the shape of the load, because that is what a catalogue, a
metadata reader and an import path actually meet:

- File sizes in the range raw files occupy. Three shapes are declared in
  `tools/corpus/src/plan.rs`, spanning 24 to 62 megabytes. Those bounds are
  stated constants chosen to cover that range. No camera was read to produce
  them and they are not a measurement of anything.
- Timestamps clustered the way a shoot clusters. Sessions are spread across the
  window the run asks for, bursts sit minutes apart inside a session, and
  frames sit a fraction of a second to a couple of seconds apart inside a
  burst.
- Filenames that collide across directories. A camera counter is four digits
  and wraps back to one, and a card fresh out of the box starts at one, so two
  cards collide from their first frame. This is the ordinary case rather than
  an exotic one, and an import path that assumes a filename identifies a
  photograph fails on it.
- Directory trees of the depth photographers produce. Half the sessions sit in
  the tree a camera writes, `card-03/DCIM/104CANON/IMG_0412.CR3`, and half in
  the tree an import produces, `library/2024/2024-06-12/session-0031/...`.
- A stated fraction of files that are truncated or malformed, because cards
  fail. Two kinds, in equal number: one runs out of bytes part way through, and
  one is its full length with opening bytes that are not what the extension
  claims. They fail a reader in different places, which is why they are two
  kinds and not one.

The damaged fraction is exact rather than sampled. The selection is a counted
step through the sorted set, so a run asking for ten per thousand of a hundred
thousand files gets a thousand of them and not approximately a thousand. A
fraction that came out near the asked-for one would be a number nobody could
quote.

## Two runs of one seed produce the same bytes

This is the claim everything else rests on. A generator that came out
differently on a second run would make every number measured against it a
number about one machine on one day.

The tool folds a digest over every path, every length and every content byte,
in path order, while it writes. Three runs of one seed, into three different
directories, with the last of them printed in full further down:

    target/release/corpus --files 100000 --seed 4242424242 --byte-divisor 2000 --out target/corpus
    digest=40c2431b701da3ce
    elapsed=443.163 s
    target/release/corpus --files 100000 --seed 4242424242 --byte-divisor 2000 --out target/corpus-second
    digest=40c2431b701da3ce
    elapsed=447.787 s
    target/release/corpus --files 100000 --seed 4242424242 --byte-divisor 2000 --out target/corpus-third
    digest=40c2431b701da3ce
    elapsed=446.736 s

`cargo run --locked --release -p corpus --` builds and runs that same binary,
which is why the command at the top of this document and the one above are the
same program. The third run was made after a change that added the conditions
block to what the tool prints and touched nothing that decides a byte, which is
why the first two show no such block and all three agree on the digest.

The digest is a Fowler-Noll-Vo fold and not a cryptographic one. It is enough
to notice that two runs differ, and it is not evidence against somebody trying
to make two different trees agree. Nothing here needs the second property.

The suite carries the stronger form at a size a test can afford: it generates
the same plan into two directories, walks both and compares every path and
every byte, rather than comparing two digests.

    cargo test -p corpus two_runs_of_one_seed_write_the_same_bytes

Both runs above are on this machine, so between them they show that the writer
holds no state from one run to the next. They cannot show that a second machine
produces the same bytes. That half is carried by a digest recorded in the suite
for one exact corpus: every machine that runs the tests folds it and compares
against the same number, so a difference in word size, in byte order or in a
path separator arrives as a red test on the machine that has it.

## What a hundred thousand files cost, measured

`--byte-divisor` is the reason there are two numbers here rather than one. A
corpus at the declared sizes does not fit on an ordinary disk, so the divisor
writes the same tree with smaller files. Its value is printed with every run,
so a set generated with one cannot be mistaken for a set generated without one.

The full-size cost, from a run that plans everything and writes nothing:

    cargo run --locked --release -p corpus -- --files 100000 --seed 4242424242 --dry-run

    conditions
        arch=x86_64
        cpus=32
        host_identity=not recorded
        os=windows
        profile=release
        byte_divisor=1
        cards=6
        dry_run=true
        files_asked_for=100000
        malformed_permille=10
        out=target/corpus
        seed=4242424242
        years=4

    corpus
        bursts=18364
        bytes_declared=3978236808835
        bytes_written=3965844584691
        damaged_bad_header=500
        damaged_truncated=500
        deepest_path=4
        directories=653
        distinct_filenames=29997
        earliest=2022-01-01
        files=100000
        files_sharing_a_filename=100000
        latest=2025-12-31
        sessions=446

    digest=not computed: a dry run writes nothing and generates nothing
    elapsed=0.117 s

Three point nine seven terabytes. That is the disk budget for a corpus at the
declared file sizes, and it is why no run at that size has been made here. The
wall-clock cost of one is not measured and nothing below should be read as
covering it.

What was measured is the same hundred thousand files with the divisor set:

    target/release/corpus --files 100000 --seed 4242424242 --byte-divisor 2000 --out target/corpus-third

    conditions
        arch=x86_64
        cpus=32
        host_identity=not recorded
        os=windows
        profile=release
        byte_divisor=2000
        cards=6
        dry_run=false
        files_asked_for=100000
        malformed_permille=10
        out=target/corpus-third
        seed=4242424242
        years=4

    corpus
        bursts=18364
        bytes_declared=3978236808835
        bytes_written=1982872006
        damaged_bad_header=500
        damaged_truncated=500
        deepest_path=4
        directories=653
        distinct_filenames=29997
        earliest=2022-01-01
        files=100000
        files_sharing_a_filename=100000
        latest=2025-12-31
        sessions=446

    digest=40c2431b701da3ce
    elapsed=446.736 s

So a hundred thousand file corpus with this shape costs 446.736 seconds of wall
clock and 1 982 872 006 bytes, which is 1.85 gibibytes, on the machine named in
the conditions above, at a divisor of two thousand. The bytes counted are the bytes written; a filesystem whose
cluster size is larger than the smaller files allocates more than that, and
that difference is not measured here.

## Where it runs in the checks

The generator's tests are ordinary tests in the workspace, so they run inside
the test gate rather than in a route of their own:

    git grep -n 'cargo llvm-cov --workspace' -- .github/workflows/tests.yml

That job prints the conditions it ran under before it runs anything: the user
and its numeric id, whether a display is set, and which device nodes exist. So
the claim that this needs no display, no elevated rights and no camera is
evidence in a log rather than a sentence in a document. The step's own comment
records that the first run of it found a DRM node present on a runner with no
display and no camera, and says so rather than rounding the result.

The corpus the checks generate is small, and the sizes are read out of the
tests rather than restated here:

    git grep -n 'build(params(' -- tools/corpus/src/

A run at that size is seconds. The hundred thousand file run is not made in the
checks and is not asked for there.

## The corpus is never committed

What is tracked is the generator, the seed and the sizes recorded above. The
corpus is not, and the default output path is what holds that: it sits under
`target/`, which `.gitignore` excludes. A test reads that declaration rather
than trusting it, so removing `/target` from `.gitignore` turns the suite red:

    cargo test -p corpus the_default_output_is_somewhere_git_ignores

The tool also refuses to write into a directory that already holds something,
and it removes nothing itself. An operator who points it at a directory with
their own files in it gets a refusal naming the directory.

## What the corpus is not

It holds no photograph. Nothing measured on it says anything about culling
quality, about focus, about faces or about whether a frame is worth keeping. It
says something about catalogue behaviour, about metadata reading and about
input and output, and only about those.

It is also not a fixture vocabulary. A test that judges a parser against bytes
this tool generated is judging it against bytes chosen to fill space, not
against a raw file any camera wrote.

## What proves the guards bite

Each of these is a one-token change somebody could actually make. Each was
applied, the suite was run, and each was reverted:

    cargo test -p corpus

- The run stops reading the seed. Three tests red, including both
  reproducibility tests.
- The damaged fraction is divided by 1024 rather than 1000. Two red.
- The damaged header is attached to the wrong defect, so no file's opening
  bytes are ever wrong. Two red.
- A dry run writes after all. One red.
- The card index is put into the filename, so no two directories ever share
  one. Two red.
