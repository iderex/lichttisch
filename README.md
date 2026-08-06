# lichttisch

Fast AI-assisted culling (sharpness, blinks, duplicates), a fluid catalogue at a hundred thousand-plus images, and tethering at professional level. It runs locally; the photographs and the culling never leave the machine.

Planning happens on the issue tracker first. Every decision that shapes
the architecture is written down there with its reasons before the code
that depends on it exists.

This project selects and catalogues photographs and never generates pixels; that
work is done in the sibling project [iderex/retusche](https://github.com/iderex/retusche),
and the boundary between them is recorded in
[docs/decisions/0007-scope-boundary.md](docs/decisions/0007-scope-boundary.md).

See [NOTICE.md](NOTICE.md) for the intended-use notice.

See [LICENSE](LICENSE) for the terms, the GNU Affero General Public License version 3.
