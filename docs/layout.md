# Repository layout

Issue: #22

Where a change belongs, without reading every module first. This note says what
each module is for and what it may not depend on. It does not enumerate the
files, the checks or the graph, because a copy of any of those in a document
drifts against the thing it describes and nothing notices.

## The modules

Each one is a crate under `crates/`, and each manifest declares its own
dependencies, so the direction below is read out of the tree rather than
promised here.

`crates/catalogue` holds the catalogue and its storage. It depends on no other
module in this workspace. Nothing above it composes a query in the storage
engine's own language, which is the rule that makes a later engine change a
change in one directory rather than a search across the tree.

`crates/foreign` holds the whole foreign-function surface and depends on no
other module. Every declaration of a foreign function and every conversion of a
foreign value into a Rust one lives here.

`crates/ingest` holds ingest and decoding. It is the only module allowed to
depend on an image library, and it reaches one only through `crates/foreign`. It
may depend on `crates/catalogue`. It may not depend on the session, the surface
or the signals.

`crates/culling` holds the culling signals. It is the only module allowed to
depend on an inference runtime, and it reaches one only through
`crates/foreign`. It may depend on `crates/catalogue`. It may not depend on the
session or the surface, because a signal that knows what the operator is doing
is a signal that can be tuned to please them.

`crates/session` holds the culling session as a headless state machine. It may
depend on `crates/catalogue` and `crates/culling`. It draws nothing and it may
not depend on the surface.

`crates/surface` holds the operator surface as a thin renderer over the session.
It may depend on `crates/session` and decides nothing. Whether this project
ships a surface of its own at all is entry 5 of issue #13 and it is open; this
module exists so that the session is written against a boundary rather than
against a window.

`crates/lichttisch` is the binary. It depends on the modules it composes and
holds no logic of its own.

## The direction, printed rather than drawn

    cargo tree --workspace --edges normal

A drawing of the graph in this file would be a second copy of the manifests, and
the copy is the one that goes stale. The command above reads the manifests.

## The foreign-function surface

Location: `crates/foreign`, and nowhere else. No other module declares an
`extern` block, and that is a rule rather than a description of what happens to
be true this week.

Size, today:

    git grep -nE '^[[:space:]]*(unsafe )?extern "' -- 'crates/**/*.rs' ; echo "exit=$?"
    exit=1

    git grep -nw 'unsafe' -- 'crates/**/*.rs' ; echo "exit=$?"
    exit=1

No foreign declaration and no unsafe block exists yet, in any module. The
decoder is not chosen, which is issue #39, and the inference runtime is issue
#50. When they arrive, the two commands above are where the size of this surface
is read, and they are written here so that a later reader gets a number rather
than an impression.

## What enforces the direction

Nothing does today, and this note says so rather than implying otherwise. The
manifests declare the direction and cargo refuses a dependency that is not
declared, so a module cannot reach another one by accident. What cargo does not
refuse is a manifest edit that adds the wrong dependency on purpose.

Issue #19 is where the boundary between the catalogue and everything that
decodes an image becomes a test, and issue #105 is where the rest of the rules
above become tests. Until those land, the direction in this note is held by
whoever reviews a manifest change.

The one guard in the workspace today is
`crates/lichttisch/tests/lock_file_is_current.rs`, and it is about the lock file
rather than about these boundaries. It is named here so that nobody reads the
existence of a test directory as the existence of a boundary test.

## Everything else

`docs/decisions/` holds one record per decision, numbered and added rather than
edited. `docs/` holds the notes that are not decisions, this one among them.
`.github/workflows/` holds the gates, and `CONTRIBUTING.md` names the command
that prints them rather than listing them.
