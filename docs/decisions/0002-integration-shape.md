# 0002 Integration shape: a program of its own

Status: accepted
Date: 2026-08-06
Issue: #3

## The question

Does this project build a culling surface inside an application that already has
a catalogue and a raw decoder, or does it build a coordinating layer of its own
that hands its result to whichever developer the photographer already uses?

The answer reaches every other issue on the board, so it is settled before
anything depends on it.

## The decision

A program of its own. A coordinating layer that holds a catalogue, does the
culling and the tethering, and hands the result to the raw developer the
photographer already uses, with the authority rule below attached to it.

## Shape one: inside an existing application

The two candidates are the raw developer and the photo manager.

The raw developer is `darktable-org/darktable`:

    gh api repos/darktable-org/darktable --jq .license.spdx_id
    GPL-3.0

Its scripting interface cannot carry this work. A Lua script there can add a
panel and an export target, and it can be told when the view changes, but it
cannot declare a view of its own. The registration entry points reachable from
Lua are for a library panel and for a storage target. Every command below is
pinned to the commit it was read at, because a line number in another project's
tree moves and an unpinned command would not reproduce:

    gh api 'repos/darktable-org/darktable/contents/src/lua/lualib.c?ref=167032086d1260c163403733f04190f06852553a' --jq .content | base64 -d | grep -n 'register_lib'
    198:static int register_lib(lua_State *L)
    301:  lua_pushstring(L, "register_lib");
    302:  lua_pushcfunction(L, &register_lib);
    gh api 'repos/darktable-org/darktable/contents/src/lua/storage.c?ref=167032086d1260c163403733f04190f06852553a' --jq .content | base64 -d | grep -n 'new_storage'
    139:static int new_storage(lua_State *L)
    171:  lua_pushstring(L, "new_storage");
    172:  lua_pushcfunction(L, &new_storage);

The `lua_pushstring` line is the name the script sees and the `lua_pushcfunction`
line is the function behind it.

A view is not among them. `dt_lua_register_view` exists, but it takes a
`dt_view_t *` and is called from C with a view that already exists, and the only
things it puts in front of a script are a name, an id and a string conversion:

    gh api 'repos/darktable-org/darktable/contents/src/lua/view.c?ref=167032086d1260c163403733f04190f06852553a' --jq .content | base64 -d | grep -n 'dt_lua_register_view\|dt_lua_type_register_const\|dt_lua_event_add'
    46:void dt_lua_register_view(lua_State *L, dt_view_t *module)
    71:  dt_lua_type_register_const(L, dt_lua_view_t, "id");
    73:  dt_lua_type_register_const(L, dt_lua_view_t, "name");
    85:  dt_lua_event_add(L, "view-changed");

That commit was the head of the default branch when this record was written:

    gh api repos/darktable-org/darktable --jq .default_branch
    master
    gh api repos/darktable-org/darktable/commits/master --jq .sha
    167032086d1260c163403733f04190f06852553a

The whole Lua surface was not read. What was read is the three files above, and
the claim is bounded by that: no route to declare a view appears in them. A
route living somewhere else in that tree would not have been seen here.

So a culling view in that tree is C inside that tree, not a script beside it.

The photo manager is `KDE/digikam`:

    gh api repos/KDE/digikam --jq .license.spdx_id
    GPL-2.0
    gh api 'repos/KDE/digikam/contents/core/dplugins?ref=33d0457e20adda97c003f3dee652a1749406ff9f' --jq '.[].name'
    CMakeLists.txt
    bqm
    editor
    generic

The first entry is the build file for the directory, not a plugin family.

The three plugin families are a batch queue action, an editor tool and a generic
menu entry. None of them is a new browsing surface, so the same conclusion
holds: the work would be C++ inside that tree.

What building inside buys is real and is not understated here. The catalogue
exists already. The decoding exists already. The photographer installs one thing
instead of two. There is one place a rating can live, so the disagreement this
decision spends a rule on cannot arise at all.

What it costs is larger. The work lands on another project's schedule and in
another project's review queue. That project's licence becomes this project's
licence rather than this project holding the terms its maintainer chose for it,
and for a copyleft neighbour that is a real constraint on everything downstream.
The catalogue engine question is answered by whatever that project already
chose, which is the question issue #5 exists to measure rather than inherit. And
a photographer who uses the other application gets nothing, which halves the
reach of a tool whose argument is that it fills a gap both of them leave open.

## Shape two: a coordinating layer

A program that holds its own catalogue, runs the culling signals, owns the
tethered capture, and writes its result where the photographer's existing tools
will read it.

It buys an engine chosen on this project's own measurement, both applications
served, terms this project sets for itself, and a frame budget no other
architecture constrains.

It costs a second catalogue in the photographer's life, and the whole
interoperability burden. That burden is why M7 is a milestone rather than a
paragraph.

## The authority rule

Two catalogues disagreeing about a rating is the failure that makes a tool like
this unusable. The rule below is the answer to it rather than a hope about it.

This project's catalogue is never the authority for anything the photographer
can also set somewhere else.

Facts this project does not own, and only caches:

- ratings
- colour labels
- rejection flags
- keywords, captions and any other descriptive metadata the photographer writes

These belong to the file and its sidecar, which is what both neighbouring
applications already read and write. A copy may be held in the catalogue for
speed, and that copy is a cache: when it disagrees with the sidecar, the sidecar
wins, and the catalogue is corrected rather than the file.

Facts this project owns, and can rebuild:

- every culling signal value and every model output
- previews and thumbnails
- group and burst structure derived from timestamps and content
- duplicate and near-duplicate relationships
- import bookkeeping such as when a file was first seen and where it was found

Facts observed from the file, which this project neither owns nor caches from a
sidecar: the bytes, the size, the digest, the embedded capture metadata. These
are read from the file and are true because the file says so.

Facts the operator sets inside this software and nowhere else: the settings of
this software itself. If a value is one the photographer could set in another
tool, it does not belong in this list, and putting it here is the mistake this
rule exists to refuse.

What survives deleting the catalogue: everything the photographer decided.
Delete the catalogue directory and nothing they set is lost, because none of it
was ever held here as the original. What is lost is the derived work, which has
to be done again. Issue #38 is where that claim is either demonstrated by a
command or found to be untrue, and issue #37 is where the same claim is tested
from the backup direction.

## What this decision does not decide

- Which catalogue engine. That is issue #5 for the measurement and issue #6 for
  the choice, and this record deliberately inherits neither.
- Which interchange format carries the cached values, and in which direction it
  is written. That is issue #10.
- Whether this project ships an operator surface of its own at all. That is
  entry 5 of issue #13 and it is open. This record decides only that the work is
  not inside somebody else's application; it does not decide that the work has a
  window. A headless coordinating layer with interoperability only is still
  shape two.
- Whether the catalogue may ever leave the host. That is entry 3 of issue #13
  and it is open.
- Whether any build may link a camera vendor kit. That is entry 2 of issue #13
  and it is open. Issue #7 places tethering without touching it.

None of the open entries above is assumed answered anywhere in this record.

## What would reverse it

Any one of the following, and each is a fact somebody can check rather than a
matter of taste:

- A neighbouring application gains a plugin surface that can declare a browsing
  view with its own key handling and its own frame timing, so that the culling
  surface could live there without the work becoming C or C++ in that tree.
- The interoperability burden proves unpayable: M7 cannot deliver a round trip
  through the sidecar that keeps the photographer's decisions intact across both
  applications, and the second catalogue therefore cannot be made tolerable by
  the authority rule.
- The maintainer settles entry 5 of issue #13 against a surface of this
  project's own and against a headless engine, at which point there is no
  coordinating layer left to place.

## Means

This record is Markdown in `docs/decisions/`, which is the means issue #3 names
in its own scope line. A decision record is read by people and diffed by git,
adds no language, runtime or dependency to the tree, and carries the commands
behind its claims as quoted text. Nothing forces another means here.
