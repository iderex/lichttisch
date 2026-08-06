# 0001 The means: language, runtime and toolchain

Status: accepted
Date: 2026-08-06
Issue: #2

## The question

Every other issue on this board assumes a language without naming one. The tree
holds documents and workflow guards and no source file at all:

    git ls-files -- '*.rs' '*.c' '*.cc' '*.cpp' '*.h' '*.py' '*.go' '*.ts' '*.js' | wc -l
    0

So the assumption is free to be wrong until it is written down. This record
writes it down, with enough reasoning that a reader can disagree on the merits.

## Three layers, and only one of them is free

The decoding layer is forced. Nobody here is writing a new raw decoder, and the
maintained ones are C and C++. `LibRaw/LibRaw` ships two licence files:

    gh api repos/LibRaw/LibRaw/contents --jq '.[].name' | grep '^LICENSE'
    LICENSE.CDDL
    LICENSE.LGPL

`darktable-org/rawspeed` is a CMake project declared in C++, and it names the
CMake version it needs:

    gh api repos/darktable-org/rawspeed --jq .license.spdx_id
    LGPL-2.1
    gh api repos/darktable-org/rawspeed --jq .default_branch
    develop
    gh api repos/darktable-org/rawspeed/commits/develop --jq .sha
    c835b05aecfacb7343f7c424abd620aa12116c3f
    gh api 'repos/darktable-org/rawspeed/contents/CMakeLists.txt?ref=c835b05aecfacb7343f7c424abd620aa12116c3f' --jq .content | base64 -d | grep -n 'cmake_minimum_required\|^project('
    1:cmake_minimum_required(VERSION 3.22) # FIXME: 3.24.2 / 3.25
    18:project(rawspeed CXX)

The command is pinned to a commit because a line number in another project's
tree moves, and an unpinned command would not reproduce. Whatever else this
project is written in, at this layer it calls a C or C++ library through that
library's own interface.

The inference layer is nearly forced in the same way and constrains the choice
much less:

    gh api repos/microsoft/onnxruntime --jq .license.spdx_id
    MIT

That runtime has bindings for every language considered below, so it rules
nothing out.

The catalogue, the ingest, the session and the surface glue are free, and that
is where nearly all of the code will be. It is also the layer that decides
whether the readme's claim about fluidity survives a hundred thousand images,
because it holds the preview cache, the query path and the frame budget at the
same time.

## The decision

Rust for the catalogue, the ingest, the session and the surface glue, calling
the decoding and inference libraries over their C interfaces from the same
process.

The floor is the Rust version this decision was taken on, and the exact pin is
issue #14's to land in a toolchain file. The version below is a fact about the
machine this record was written on rather than a property of this repository,
which holds no toolchain file yet:

    rustc --version
    rustc 1.97.0 (2d8144b78 2026-07-07)
    cargo --version
    cargo 1.97.0 (c980f4866 2026-06-30)

    gh api repos/rust-lang/rust/releases --jq '.[0].tag_name'
    1.97.1

The build host also needs a C++ compiler and CMake at the version the decoder
names above. That is a build-host cost, not an operator cost: one binary ships
and the operator installs nothing underneath it.

## Can the means carry the three rules

A property a machine can refuse. Rust refuses at compile time what this project
would otherwise refuse at review time, and the properties this board cares about
beyond that are greppable or testable rather than type-level. Nothing in the
choice makes a refusal harder to write than in the alternatives.

A proof that runs. `cargo test` is in the toolchain rather than beside it, so a
test harness is not a thing to choose, install and keep alive. Issue #17 owns
the harness and the coverage floor and inherits a runner rather than building
one.

A claim that cites the command behind it. This is a property of how the work is
written and no language supplies it. What the toolchain supplies is that the
commands a reader would run are few and are the same on every machine: one build
command, one test command, one lock file.

## Is anything outside this repository forcing it

Yes, at one layer, and the force is real, named and held to its smallest
surface. The decoders are C and C++ and will not be rewritten, so the boundary
to them is not a choice. What is a choice is how much of the program lives on
the far side of it, and the answer here is as little as possible: the foreign
side decodes bytes and returns pixels and metadata, and every other decision
about a photograph is made in Rust.

Nothing forces the free layer. The neighbouring applications are C and C++
because they began before the alternatives existed, which is a fact about their
history rather than an argument about this project's future.

## What it adds to the tree, and whether that cost is paid knowingly

The tree today carries no language, no runtime and no dependency. Whatever is
chosen adds the first of each, so this question is answered against the
alternatives rather than against zero.

What Rust adds that C++ throughout would not: a binding layer to two C and C++
libraries, which has to be written and kept honest, and a mistake there is a
crash in a process holding somebody's archive rather than an exception. That
cost is accepted here rather than argued away, and three things answer it. The
surface is bounded and named below. Issue #40 owns proving it stays small.
Issue #101 points the fuzzing at it first, because the bytes crossing it arrived
on a card from somewhere.

What it adds for the operator: nothing. One artefact, no runtime to install
first, no interpreter version to match.

## Is it testable by the suite that will exist

The suite that will exist is the one issues #17, #52 and #106 describe, and it
is a single runner over one workspace. Nothing in this choice needs a second
apparatus beside it: the foreign side is exercised through the same tests as the
Rust that wraps it, because the wrapper is what the tests call. Where the
foreign side needs its own fixtures, those are files rather than a second
harness.

The one place this would have gone wrong is a split language stack, where the
catalogue tests and the session tests run under two runners and the coverage
figure is two figures. That is the shape this decision avoids rather than the
shape it creates.

## The foreign-function surface

Location, so a later reader finds it without searching. Every declaration of a
foreign function, and every conversion of a foreign value into a Rust one, lives
under one directory of the workspace, and issue #22 names it in the layout note.
No other directory declares an `extern` block, and no other directory is
permitted to.

Size, stated as a bound rather than as a measurement, because there is nothing
to measure yet. The surface holds the decode entry points and the inference
session entry points and nothing else. It carries no catalogue type, no session
type and no query. What crosses it is bytes in one direction and pixels,
dimensions and capture metadata in the other.

That bound is a sentence until a machine refuses a violation of it. Issue #40 is
where the refusal is built and where the size becomes a number produced by a
command. Until then this paragraph is prose, and it is written here so that #40
has something to refuse against.

## Alternatives, and what each one costs

C++ throughout buys direct use of both libraries with no binding layer at all,
and it is what the comparable applications are written in. It costs the
memory-safety argument in a process whose whole job is to parse files that
arrived from somewhere, and it costs the lock file and the dependency review
story that the guards already in this tree assume.

Python for the catalogue and the session, with C++ underneath, buys the fastest
route to a working culling signal and the ecosystem the models already live in.
It costs the frame budget, which is decided at exactly this layer, and it puts a
garbage collector between the shutter and the screen during tethered capture.

Go buys a simple toolchain and easy concurrency. It costs a foreign-call
boundary on the hottest path in the program, which is per-image decoding.

None of the three is disqualified by the three rules, and this record does not
pretend otherwise. Each is a trade, and the trade above is the one taken.

## What would reverse it

The catalogue measurement in issue #5 shows the work is so thoroughly disk bound
that the language cannot be read in the numbers. Then the cheaper toolchain is
the better answer, and this record is superseded by a new one rather than
edited.

The binding layer proves unpayable in practice: issue #40 cannot hold the
surface to the bound above without the wrapper growing into a second
implementation of what it wraps.

The decoder decision in issue #39 lands on a candidate whose only usable
interface is C++ rather than C, at a shape no thin binding reaches. That does
not reverse the language of the free layer on its own, and it is written here so
that a later reader does not read a C++ binding as a breach of this record.

## What this record does not decide

- Which raw decoder. That is issue #39, and the licence compatibility argument
  the AGPL choice forces on it is that issue's to make, not this one's.
- Which camera transport, and the same compatibility argument. That is issue
  #74.
- Which catalogue engine, or the language its client is written against. That is
  issue #5 for the measurement and issue #6 for the choice.
- Whether this project ships an operator surface of its own at all. That is
  entry 5 of issue #13 and it is open. This record names the glue that a surface
  would be written against and does not decide that there is one.
- What the surface is built with, if there is one. A surface toolkit is a means
  choice of its own, made against the standpoint when the artefact is built,
  rather than inherited from this record.

## The convention every later record follows

Naming: `docs/decisions/NNNN-slug.md`, four digits, the number never reused, the
record added rather than edited once it is accepted. Superseding a record is a
new record that names the one it replaces.

Every record answers the four questions above for its own artefact, in a section
named `## Means`. Every record in the directory carries that section, including
the five that landed before this one:

    git grep -l '^## Means' -- docs/decisions/ | wc -l
    6
    git ls-files -- docs/decisions/ | wc -l
    6

What those five earlier sections answer in full is the third question, the one
about what the artefact adds to the tree. The other three are
answered by the shape of the artefact rather than sentence by sentence, because
a Markdown document in a directory of Markdown documents forces no means, adds
no runtime and needs no harness. This is stated rather than smoothed over: the
convention going forward is all four questions, and it is not being claimed
retrospectively of records that predate it.

Nothing reads any of this. No check in this tree opens a decision record, and
the numbering, the section and the four questions are held by whoever reviews
the change. That gap is what issue #105 would close for the architecture rules
and what nothing closes for this one today.

## Means

This record is Markdown in `docs/decisions/`, which is the means issue #2 names
in its own scope line. A decision record is read by people and diffed by git,
adds no language, runtime or dependency to the tree, and carries the commands
behind its claims as quoted text. Nothing forces another means here.
