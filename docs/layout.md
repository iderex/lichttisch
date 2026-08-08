# Repository layout

Issue: #22

Where a change belongs, without reading every module first. This note says what
each module is for and why the direction between them runs the way it does. It
does not enumerate the files, the checks or the graph, and since #19 it does
not carry the permission list either: `crates/module-boundaries.txt` holds
that, and a test refuses it when it stops being true. A copy of any of those in
a document drifts against the thing it describes and nothing notices.

## The modules

Each one is a crate under `crates/`. What each may reach is on its own line in
`crates/module-boundaries.txt`, with the reason on the lines above it, so the
paragraphs here say what a module is for rather than repeating what it is
allowed to touch.

`crates/catalogue` holds the catalogue and its storage. Nothing above it
composes a query in the storage engine's own language, which is the rule that
makes a later engine change a change in one directory rather than a search
across the tree. It parses nothing that arrived from a card, a camera or a
stranger, and that is the boundary #19 is about.

`crates/foreign` holds the whole foreign-function surface. Every declaration of
a foreign function and every conversion of a foreign value into a Rust one
lives here.

`crates/ingest` holds ingest and decoding. It is the only module allowed to
reach an image library, and it reaches one only through `crates/foreign`, which
is what keeps a fuzzer pointed at decoding pointed at something.

`crates/culling` holds the culling signals. It is the only module allowed to
reach an inference runtime, and it reaches one only through `crates/foreign`. It
stays clear of the session, because a signal that knows what the operator is
doing is a signal that can be tuned to please them. The shape every signal is
built against, and the one route a signal is called through, are here rather
than in each signal, and `docs/decisions/0012-signal-interface.md` is where that
shape is argued.

`crates/session` holds the culling session as a headless state machine. It
draws nothing, and it stays clear of the surface so that the thing holding the
state cannot depend on the thing drawing it.

`crates/surface` holds the operator surface as a thin renderer over the session,
and decides nothing. Whether this project ships a surface of its own at all is
entry 5 of issue #13 and it is open; this module exists so that the session is
written against a boundary rather than against a window.

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

`crates/lichttisch/tests/module_boundaries.rs`, against the declaration in
`crates/module-boundaries.txt`.

The rule it holds is the one #19 states. A module may reach only the packages
its own line names, whether the dependency is written in that module's manifest
or arrives through another one. The catalogue's line is empty, which is how "no
image library and no metadata library, directly or through anything else" is
said in a form something can refuse. A member with no line at all fails rather
than passing by default, so a new module cannot arrive unplaced, and a line
naming a member the workspace no longer has fails as well, so a permission does
not outlive the module it was written for.

What that adds to what cargo already does is the part worth being exact about.
Cargo refuses a dependency that is not declared, so no module reaches another
one by accident. It has nothing to say about a manifest edit that adds the
wrong dependency on purpose, and nothing at all to say about a library that
arrives three levels down behind a feature somebody turned on.

The test does not read the manifests itself. It asks `cargo tree` what the
resolver actually reaches, with dev and build edges excluded, every feature
enabled and every target included, because a dependency that is optional today
or declared for one platform only is still a dependency of the module that
carries it.

The bound, stated rather than left to be found: a line may permit a package
that nothing reaches any more, and nothing refuses a permission that has gone
stale.

## What enforces the rest

The rules this note states that are not about the dependency direction, the
foreign-function surface among them, are `crates/lichttisch/tests/architecture_rules.rs`.
Which of them that file refuses is printed rather than written down:

    cargo test -p lichttisch --test architecture_rules -- --list rule::

The rules it does not refuse are in `docs/architecture-rules.md`, each with the
reason nothing can judge it today and what would. Between the command above and
that document every structural rule in this tree is accounted for, and a rule
appearing in neither is the failure both halves exist to make visible.

`crates/lichttisch/tests/lock_file_is_current.rs` is the other guard in the
workspace, and it is about the lock file rather than about these boundaries.

## Everything else

`docs/decisions/` holds one record per decision, numbered and added rather than
edited. `docs/` holds the notes that are not decisions, this one among them.
`.github/workflows/` holds the gates, and `CONTRIBUTING.md` names the command
that prints them rather than listing them.
