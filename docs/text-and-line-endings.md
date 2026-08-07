# Text, line endings and exact bytes

Issue: #23

This project will assert on what a parser does with specific bytes. That makes
the handling of text on the way into git a correctness question rather than a
matter of taste, so it is declared rather than inherited.

## What is declared

`.gitattributes` holds the declaration. The default line is:

    * text=auto eol=lf

Git decides what is text, and every text path is stored and checked out with
LF. A contributor's `core.autocrlf` no longer decides what a commit contains,
which is the point: the same working tree produces the same blob on every
machine.

Three other groups are declared there, each with its reason on the spot:
a fixture directory declared binary so its bytes are never rewritten, the image
and model formats declared binary so git does not have to guess from the first
few bytes, and a place for carriage-return exceptions, of which there are none
today.

## What the guard catches, and what it does not

`.github/workflows/line-endings.yml` fails when a tracked text file carries a
carriage return in the blob git stores.

It is worth being exact about what that leaves, because the obvious case is not
one of them. `* text=auto eol=lf` removes a CRLF pair on the way into git, so a
file saved by a Windows editor is normalised before it is ever stored and the
guard never sees it. Measured rather than assumed:

    printf 'title\r\nbody line\r\n' > docs-note.md
    git add docs-note.md
    git show ":docs-note.md" | od -c | head -2
    0000000   t   i   t   l   e  \n   b   o   d   y       l   i   n   e  \n
    0000020

The carriage returns are gone from the blob. A guard written against that case
alone would be a guard that cannot fail.

Two cases do reach a blob, and they are what the guard is for.

A lone carriage return. Normalisation rewrites a CRLF pair and leaves a bare CR
alone, so a value pasted out of a terminal that overwrote its own line, or a
file with old Mac line endings, is stored exactly as it arrived. It looks
correct in an editor and it looks correct in the rendered diff:

    printf 'the value was 12\rthe value was 13\n' > note.md
    git add note.md
    git show ":note.md" | od -c | head -2
    0000000   t   h   e       v   a   l   u   e       w   a   s       1   2
    0000020  \r   t   h   e       v   a   l   u   e       w   a   s       1

and the guard refuses it:

    FAIL    note.md carries a carriage return in the blob git stores
    tracked=6 binary=2 exempt=0 examined=4
    ::error::A tracked text file carries a carriage return in the blob git stores.

with the carriage return stripped and nothing else changed:

    tracked=6 binary=2 exempt=0 examined=4
    No carriage return in any examined text blob.

A blob that entered the tree before the declaration covered it. Storage has
already happened and nothing normalises it afterwards. In a tree where the file
was added first and the declaration second:

    printf 'legacy\r\nfile\r\n' > legacy.md && git add legacy.md
    git show ":legacy.md" | od -c | head -2
    0000000   l   e   g   a   c   y  \r  \n   f   i   l   e  \r  \n
    0000016

    FAIL    legacy.md carries a carriage return in the blob git stores
    tracked=2 binary=0 exempt=0 examined=2

Both proofs were run in a scratch repository rather than against this tree,
because committing a deliberately broken file here to watch a check go red
would leave that commit in the history the guard is meant to keep clean.

## The exception mechanism

A path declared `eol=crlf` in `.gitattributes` is exempt. The guard asks git for
that attribute rather than holding a list of its own, so the declaration and the
exemption cannot drift apart. Every such line carries its reason on the line
above it.

Proven in both directions on the second tree above. With the declaration added:

<!-- docs-lint: illustrative, what the guard printed rather than a command -->

    exempt  legacy.md (.gitattributes declares eol=crlf)
    tracked=2 binary=0 exempt=1 examined=1
    No carriage return in any examined text blob.

With it withdrawn and nothing else changed:

    FAIL    legacy.md carries a carriage return in the blob git stores
    tracked=2 binary=0 exempt=0 examined=2

There are no exceptions in this tree today:

    git ls-files | git check-attr --stdin eol | grep ': eol: crlf$'

## The interaction with the unicode guard

`.github/workflows/unicode-guard.yml` rejects bidirectional and invisible
Unicode control characters. It reads the working tree, this guard reads the
stored blob, and their character sets do not overlap: the unicode guard's set is
the bidi overrides, isolates, marks and the zero-width characters, and a
carriage return is in neither its set nor its intent.

They do interact, in one direction, and it is a cost rather than a coincidence.
Both use `git grep -I`, and `-I` treats a path declared `-text` as binary. So
declaring the fixture directory binary removes it from the unicode guard's scan
as well as from this one. Measured:

    printf 'left to right override here: \342\200\256 and more text\n' > tests/fixtures/bidi-sample.txt
    git add -A
    git grep -nIP '(*UTF)[\x{202A}-\x{202E}\x{2066}-\x{2069}\x{200E}\x{200F}\x{061C}\x{200B}-\x{200D}\x{2060}]' -- .
    tests/fixtures/bidi-sample.txt:1:left to right override here: ... and more text
    exit 0, a match

and with `tests/fixtures/** -text -diff` declared and nothing else changed:

    git grep -nIP '(*UTF)[\x{202A}-\x{202E}\x{2066}-\x{2069}\x{200E}\x{200F}\x{061C}\x{200B}-\x{200D}\x{2060}]' -- .
    exit 1, no match

The matched line is elided above: a document holding the literal character would
be refused by the guard that reads this tree. The command reconstructs it.

So a bidi override inside a file under `tests/fixtures/` is not seen by the
unicode guard. That is stated here rather than left to be discovered, and it is
the reason the convention below prefers not to put fixture bytes in a file at
all. A fixture directory that has to hold real files is still the right answer
for a whole raw file or a fuzz corpus, and the blind spot is the price.

## The convention for exact bytes in a test

Exact bytes belong in the source, base64-encoded, decoded by the test. Not in a
file the test reads.

The reason is the one this document opens with. A raw literal in a source file
is subject to every normalisation rule above, and the carriage return a fixture
exists to prove is exactly the byte that normalisation deletes. Base64 is
ordinary text under every rule here: nothing rewrites it, the unicode guard
still reads the file it lives in, and the bytes the test sees are the bytes
somebody wrote.

Where a fixture genuinely has to be a file, because it is a whole raw file or a
fuzz corpus rather than a byte sequence, it goes under `tests/fixtures/`, which
is declared binary so no byte is rewritten. That directory carries the unicode
guard blind spot measured above, so it is the second choice rather than the
default.

## The first fixture

`crates/lichttisch/tests/module_boundaries.rs` is the first user, and it did not
arrive to demonstrate the convention. It arrived because the module boundary
declaration is parsed, and what the parser does with one byte turned out to
matter.

`str::lines` splits on the line feed alone. A lone carriage return is therefore
not a line break to that parser, while an editor honouring it draws one. So

<!-- docs-lint: illustrative, a line of the boundary declaration under discussion -->

    catalogue ->

followed by a carriage return and then `foreign` is two lines on the screen,
the first of which grants nothing, and one line to the parser, which reads it
as the catalogue being permitted to reach the foreign-function surface. A
permission nobody wrote, arriving from a byte nobody sees. The parser refuses a
carriage return rather than trimming it, and the fixture is what proves the
refusal.

The fixture is two base64 constants a byte apart, decoded in the test: the text
above with the carriage return, and the same text with a space in its place.
Three legs, and the first is why this is in the source rather than in a file.
One leg asserts the decoded bytes still carry a lone carriage return, so a
fixture that lost the byte on the way in cannot leave the pair looking green.
One asserts the parser refuses it. One asserts the neighbour, one byte away, is
read.

Measured by deleting the guard and leaving everything else alone:

<!-- docs-lint: illustrative, the message the test printed when the guard was gone -->

    a lone carriage return was read as ordinary text, so a line the file
    appears to withhold was granted: Ok({"catalogue": {"foreign"}})

and by swapping the fixture's own constant for its neighbour's, which is the
mistake that would make the pair prove nothing:

<!-- docs-lint: illustrative, the message the test printed against the wrong fixture -->

    the fixture is meant to carry one carriage return that is not part of a
    pair, and it carries "catalogue -> foreign\n"

The base64 helper in that file is where the second fixture should start. It
decodes the standard alphabet and nothing else, and it fails on a byte outside
it rather than skipping one.
