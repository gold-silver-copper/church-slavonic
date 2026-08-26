# old-church-slavonic-core

> **Succession notice.** This crate's consumer surface is superseded by
> the rule-first [`church-slavonic`](../church-slavonic) facade (and
> `church-slavonic-dictionary`); see `docs/DEPRECATION_MAP.md` for the
> item-by-item mapping and `docs/REWRITE_PLAN.md` for the program. This
> crate remains the reference implementation until the final deprecation
> release is published.

Pure, rule-based Old Church Slavonic morphology. This crate bundles no dictionary,
performs no I/O, and returns typed failures when a lemma does not carry enough
lexical information. Use `old-church-slavonic` for dictionary-backed forms.

Grammar, result, and trace types have explicit root re-exports. Productive entry
points remain organized by linguistic owner in `noun`, `adjective`, and `verb`;
callers supply the corresponding typed lexeme and cell. A citation alone never
silently selects a lexical class or principal part.

Verb generation requires independent typed principal parts for the present,
imperfect, aorist, imperative, l-participle, and each non-l participle system.
Lexical aspect never chooses an aorist. The core supports explicit A/Yat-A/
palatalized and present-stem imperfect platforms under explicit uncontracted,
contracted, or iotated variant policies;
asigmatic, new *ox*-, old `-с-`, old `-х-`, and vowel-stem sigmatic aorists;
both historical imperative series; and audited participial seams including
transformed i-stem `-ьш-`, declared final-j deletion, and `ov → u`. Every
sigmatic analysis carries an explicitly graded main stem and an independent
complete 2sg/3sg principal part, so zero, `-тъ`, and `-стъ` variants remain
separate lexical analyses rather than guessed combinations.
