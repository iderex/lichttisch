# The terms, and what keeps every statement of them agreeing

Issue: #110

`LICENSE` is the terms. Everything else in this tree that says anything about
them is a pointer at that file, and a pointer that stops agreeing with what it
points at is worse than no pointer, because it is read and acted on.

So there is one identifier here, it is written the same way everywhere, and a
test refuses each departure from it that a reading of the tracked tree can see.

## The identifier

    AGPL-3.0-only

That is the SPDX short form for the licence in `LICENSE`, and it is what the
workspace manifest declares, what every member inherits, what the dependency
policy permits, and what every source file opens with.

The hosting platform answers with the older identifier for the same licence:

    gh api repos/iderex/lichttisch --jq .license.spdx_id
    AGPL-3.0

Both name one licence. Carrying both is how a tree comes to have two answers to
one question, so documents written after this note state the one above, and a
document quoting the platform's answer quotes it inside the block under the
command that produced it rather than in a sentence of its own.

## Why every source file carries it

A file copied out of this tree carries no terms unless it carries them itself.
That is the whole reason, and it is not hypothetical: a module lifted into
another project, a snippet pasted into an issue, a file recovered from a build
directory. The identifier is the one line that travels with the file.

It goes on the first line. A licence scanner reads the head of a file, a person
opening one sees the head first, and one position has one answer where "near the
top" has as many answers as there are readers.

## Why the short form rather than a notice block

The licence itself suggests a notice naming the program and the copyright
holder in every file. The short identifier is what this tree carries instead,
for two reasons that are worth stating rather than leaving as taste.

The first is that a notice block is a paragraph, and a paragraph gets edited
into several slightly different paragraphs across a tree. The identifier is one
line with one correct spelling, which is why a machine can refuse a wrong one.
The full grant, the warranty disclaimer and the address of the licence stay in
`LICENSE`, in one copy, unedited.

The second is that the copyright holder is a fact about people rather than about
files. Writing it into every file means editing every file the day it changes,
and a tree that carries it in one place has one edit to make.

What this costs is that a file travelling on its own carries a pointer rather
than the full notice. The pointer is unambiguous, which is what the short form
is for.

## What is checked

    cargo test -p lichttisch --test licence

Five statements, each with its own leg, and each leg is a pure function over
text with a near-miss beside it in the same file.

Every tracked Rust source opens with the identifier. The subjects come from
`git ls-files` at the commit under test, so a module added tomorrow is judged
tomorrow rather than being absent from a list somebody forgot to extend.

Every workspace member takes its terms from the workspace. A member writing the
field out is refused even when the value is right, because two homes for one
fact is the state that lets them disagree, and the disagreement is invisible in
a build.

The workspace declares the identifier, and `deny.toml` permits it. A tree whose
own terms are missing from the allow list its own dependency gate reads would
refuse itself as a dependency.

`LICENSE` is the licence the identifier names. This is the one leg that reads
what the pointer points at. Every other leg compares the tree against the
identifier, so an identifier sitting over the text of a different licence would
survive all of them.

`README.md` names that licence by the title the file carries and points at the
file, and no document above the record watermark states the terms in the older
spelling outside a block.

## What is not checked, and why

Whether the terms are the right terms for a given file. A file copied in from
somewhere else, under somebody else's licence, passes the moment it carries this
header, and it passes wrongly. Nothing in a reading of the text sees where a
file came from. That one is held by whoever reviews the change that adds it, and
it is the reason this note does not present a green run as a licensing review.

Untracked files. They are invisible to every leg here, which is the correct
blind spot: an untracked file is not part of what is distributed.

Prose in the decision records that landed before the rule. A record is added
rather than edited once it is accepted, so a rule reaching backwards would make
a landed record permanently red with no legal repair. The watermark is in
`crates/lichttisch/tests/licence.rs` with the record it exempts named, and the
same device is used for the same reason in
`crates/lichttisch/tests/architecture_rules.rs`.

The pasted answer of a command. Refusing the platform's own spelling where a
command returned it would refuse the evidence rather than the claim, and that
teaches somebody to delete the paste instead of fixing the sentence.

Non-Rust sources. There are none in this tree today. A manifest, a workflow file
and a declaration file state terms by inheriting them or by not being source at
all, and the day a second language arrives is the day this rule gains a second
subject and its own near-miss.

## What a contributor grants

`CONTRIBUTING.md` states it, and `DCO.md` is what the sign-off trailer asserts.
The two are different halves and this note keeps neither: one is what you
certify about the origin of the work, the other is the terms the work is
contributed under.
