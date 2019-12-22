# lilv-rs

This is a Rust wrapper for [Lilv](http://drobilla.net/software/lilv),
the LV2 host library.

**Please be cautious when using this crate!** It may work or break;
for the moment its intended use is as a dependency for a certain project.
It attempts to wrap everything nicely in idiomatic ways, but all functionality
is not tested.

## Completeness

This crate targets the latest version of Lilv, which is at the time of
writing, 0.24.2.

When rewriting this, I lost count of which individual functions are wrapped
and which ones are not. `src/state.rs` is empty, that's all I can say for
sure.

## Notes

### 2019-12-22

As I said in the beginning, this crate was primarily in use by another project.
However, even then I did rewrite it in that project's own repository, leaving this
repository rotting. I've now updated this repo with that code.

The other project uses [`parking_lot`] locks exclusively, and as the new code was
admittedly written only with that project in mind, so does this crate now
for consistency. That could be moved behind a feature gate if desired.

[`parking_lot`]: https://docs.rs/parking_lot
