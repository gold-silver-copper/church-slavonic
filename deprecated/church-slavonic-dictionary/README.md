# church-slavonic-dictionary (deprecated)

**This crate is deprecated and will receive no further releases.** Version
`0.2.1` is an empty final release published only to carry this notice.

It held Old Church Slavonic senses and lemmatisation over the rule-first facade; senses are now deterministic _n keys in the church-slavonic tables.

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
