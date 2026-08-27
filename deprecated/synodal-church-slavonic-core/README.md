# synodal-church-slavonic-core (deprecated)

**This crate is deprecated and will receive no further releases.** Version
`0.6.1` is an empty final release published only to carry this notice.

It held the Synodal Church Slavonic rule engine; those rules now live in church-slavonic-core as the Synodal recension.

It is superseded by [`church-slavonic`](https://crates.io/crates/church-slavonic)
`0.4` (facade, with the generated tables) and
[`church-slavonic-core`](https://crates.io/crates/church-slavonic-core) `0.4`
(the rule engine), which cover both Old Church Slavonic and Synodal Church
Slavonic through one API selected by a `Recension` argument:

```toml
[dependencies]
church-slavonic = "0.4"
```

See the repository README for the architecture, accuracy tables and
migration notes: <https://github.com/gold-silver-copper/church-slavonic>.

The last functional release of this crate remains available on crates.io at
the previous version; pin it with an exact requirement (`=x.y.0`) if you
need it. Its sources are in the repository's git history (tag
`pre-english-parity`).

Licensed under MIT OR Apache-2.0.
