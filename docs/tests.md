# The suite, the floor and the tests that cannot run here

Issue: #17

The suite exists before most of the code it will judge, so that the first
feature arrives with a place to put its test rather than a reason to defer one.

## The command

One command line runs every test and measures what it reached. It is not copied
into this document, for the reason `CONTRIBUTING.md` gives about every other
gate: a second place holding a gate's logic is a second place that goes stale.
Read it out of the workflow that owns it.

    sed -n '/run: |/,$p' .github/workflows/tests.yml

The coverage runner has to be installed first, and the workflow names the exact
version. `rust-toolchain.toml` carries the `llvm-tools` component the runner
needs, so a fresh clone has it without anybody choosing it.

## Why the command has two halves

The coverage runner executes the compiled tests. It cannot execute doctests on
a stable toolchain, so the second half is a plain `cargo test --doc`.

That is a real gap and it is written here rather than hidden: a doctest runs,
and fails the gate when it fails, but the lines it exercises are not counted
towards the floor. There are none today:

    cargo test --locked --workspace --doc
    running 0 tests

## The floor, and what it is not

The number and its reason are in `.github/workflows/tests.yml`, on the step
that uses it, because the command carries the number and a second copy here
would be the drift this project keeps refusing.

What matters more than the number is what it does not do. It is one total over
the whole workspace, because the runner reports one total. A module arriving
with no tests at all therefore hides behind a well covered neighbour until it
is large enough to move that total, which is exactly the case the floor was
described as catching. So the floor catches it late rather than immediately,
and nothing here catches it early.

Coverage is also not a measurement of whether the tests are any good. A line
executed by a test that asserts nothing counts the same as a line whose
behaviour is pinned. Issue #100 is where the tests are judged by mutation
rather than by this number, and reaching the floor should be read as saying
nothing flattering at all.

## What the check's environment actually is

The gate prints the conditions it ran under rather than asserting them, and the
printed answer is not uniformly the convenient one. On its first green run the
step reported a non-root user, no display, no Wayland display, no camera node,
no NVIDIA node, no `/dev/kfd` and no accelerator tool on the path, and one DRM
node present:

    user=runner uid=1001
    display=unset wayland=unset
    devices /dev/video*: none
    devices /dev/dri/*: /dev/dri/card1
    devices /dev/nvidia*: none
    devices /dev/kfd: none
    nvidia-smi=absent

So the run is unelevated, without a display and without a camera, and a device
node exists that nothing in this project reaches. That is written down rather
than rounded off, because the claim a later reader will want to lean on is
about what the suite needs, and the log is what they will check it against.

## A test that cannot run here

Some tests will need a camera on the end of a cable, an accelerator, or a
person to watch what happens. None of those exists in the automated checks.

Mark such a test and say what it is waiting for:

    #[ignore = "hardware: needs a camera on the other end of a cable"]
    #[ignore = "manual: a person has to look at the result"]

The prefix is the convention and it is enforced rather than asked for.
`crates/lichttisch/tests/ignore_markers.rs` reads the tracked sources and
refuses an ignore whose reason does not open with one of those two words.

The reason a bare `#[ignore]` is refused is not tidiness. It reads as "skipped"
in the run output with nothing saying what would have to be true for it to run,
so a test somebody turned off while debugging looks exactly like a test waiting
for hardware, and the first one never comes back on.

The default run skips them and says how many it skipped, so a run that measured
less than the whole suite cannot be read as one that measured all of it. To run
them where the hardware is:

    cargo test --locked --workspace -- --ignored

The guard reads the sources as text, so an ignore produced by a macro is
invisible to it, and so is one in a file git does not track.
