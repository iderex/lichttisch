# What this software is for, and what it is not built for

Issue: #115

Face detection over an archive of photographs is a capability with obvious
misuses, and a project that stays silent about them has made a choice. This
document is the specific statement. `NOTICE.md` carries the general one and
links here rather than repeating any of this.

What follows is not a disclaimer. A disclaimer asks an operator not to do
something the software would happily do. Where this project can say instead that
the software does not do a thing and holds nothing that would let it, that is
worth more, and it is said here only where it is true. Where the property rests
on a document rather than on a mechanism, or on code that has not been written
yet, this says so in those words.

## What it is for

Culling. Reducing a shoot to the frames worth keeping, on photographs the
operator already holds, on the operator's own machine.

The signals it computes are about the picture: whether it is sharp where the
subject is, whether the exposure is recoverable, whether the eyes in it are
open, and which frames in a burst are the same picture. What each signal is and
where it comes from is `docs/decisions/0006-signal-sources.md`.

The operator decides. `docs/decisions/0005-verdicts-and-error-cost.md` is where
the difference between a machine's opinion and a photographer's decision is set
out, and this software never converts the first into the second.

## What it is not built for

Identifying a person. Nothing here answers who is in a photograph, and no part
of the plan asks that question.

Surveillance. Nothing here is built to observe people, to watch a place, or to
run over somebody else's photographs on somebody else's behalf.

Building a record of who appears where. No face index, no per-person timeline,
no search by person, and no export shaped to feed one.

Those three are not requests. They are what the rest of this document is about.

## What in the software makes those unsupported

Three different strengths of statement, kept apart because collapsing them is
how a document ends up promising more than the tree delivers.

### Held by a test today

The dependency direction between the modules is declared in
`crates/module-boundaries.txt` and refused by
`crates/lichttisch/tests/module_boundaries.rs`. Two of its lines matter here.
The culling signals may not reach the session, so a signal cannot know what the
operator is doing. The catalogue's line is empty, so the module that answers
queries reaches no image library and no metadata library, directly or through
anything else.

The structural rules are refused by
`crates/lichttisch/tests/architecture_rules.rs`, and the ones it holds that bear
on this are the confinement of every foreign-function declaration to
`crates/foreign` and the absence of any dependency here on the neighbouring
project. The list is the test names rather than a copy of them:

    cargo test -p lichttisch --test architecture_rules -- --list rule::

What none of those tests do is refuse a feature that identifies somebody. They
are about which module may reach which, not about what a module computes. Read
as protection against identification they would be read wrongly, and that is
why they are listed with what they actually hold.

### Held by a decision record, and by nothing else

No identity is attached to a face. `docs/decisions/0010-catalogue-schema.md`
puts "who is in a photograph" under what the schema deliberately does not model:
faces are found and measured, and nothing about a person is stored. Nothing
refuses a column that would break that, because there is no schema in the tree
yet. Issue #27 is where the model becomes migrations, and issue #113 is where
what is computed, kept and deleted about a face is stated.

No photograph reaching this software trains anything.
`docs/decisions/0006-signal-sources.md` states that rule and states in the same
paragraph that it is enforced nowhere: it holds today because no training code
exists in the tree, which is a fact about the tree rather than a mechanism.
Issue #102 is the check that would refuse it.

The face models are used as published, and what is computed from their output is
arithmetic this project states rather than a model it fits. That is in the same
record, and it is the reason there is no fitted model here to be repurposed.

### True only because the code does not exist yet

The culling module holds no code. Neither does the module that would hold the
foreign surface, nor the session. Each is a manifest and a file of doc comments
saying what will go there:

    git ls-files crates/culling crates/session crates/foreign

Every property above about what a signal computes is therefore a property of a
plan at the time of writing, not of a program. A reader deciding whether to
trust this document should weigh that first. What is written here is what the
tree will be held to as those modules fill, and each of the issues named above
is where that happens.

## Whose photographs, and on what basis

This software processes photographs the operator already lawfully holds. It has
no other source: it reads a folder the operator points it at, and it takes
frames from a camera the operator has connected.

Nothing in it establishes a lawful basis the operator does not already have.
Running a photograph through this software does not create a right to hold that
photograph, to process what is in it, or to keep what was computed from it. If
the operator has no basis for holding the picture, this software gives them
none, and every legal question about the pictures is the same question it was
before the software was involved.

The data protection statement is issue #112 and is not written yet. When it
lands it says what is kept and where; this document says what the software is
for.

## What this document does not claim

It does not claim the software cannot be modified to do any of the things above.
It is free software under `LICENSE`, somebody may change it, and no sentence
here reaches a fork.

It does not claim that a face measurement is anonymous. A measurement taken from
a face is data about a person even when no name is attached, which is why issue
#113 exists rather than being answered by the paragraph above.

It does not claim anything about where data goes. Whether the catalogue may ever
leave the host is entry 3 of issue #13, it is open, and issue #116 is where a
crossing becomes a deliberate act with its own door. Until both are settled,
nothing in this document should be read as a statement about a network.

It does not claim a guarantee anywhere the previous section did not give one.
The three headings above are ordered by strength deliberately, and the last of
them is the largest.
