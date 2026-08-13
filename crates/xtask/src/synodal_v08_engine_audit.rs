use std::{error::Error, fs, path::Path};

const MATRIX: &str = "data/synodal/engine_capabilities.tsv";
const OUTPUT: &str = "docs/SYNODAL_V10_PRODUCTIVE_MORPHOLOGY_AND_LEXICON_AUDIT.md";
const HEADER: &str = "category\tsubtype\trule_id\ttarget_recension\tstatus\tvalid_cells\tinvalid_cells\trequired_metadata\talternations\taccent_contract\tsource\tcitation\tgolden\tboundary\timplementation\ttest\tfailure";

#[derive(Clone, Debug)]
struct Capability {
    fields: Vec<String>,
}

impl Capability {
    fn get(&self, index: usize) -> &str {
        &self.fields[index]
    }
}

pub(crate) fn run(
    args: &mut impl Iterator<Item = String>,
    root: &Path,
) -> Result<(), Box<dyn Error>> {
    let mut check = false;
    for argument in args {
        match argument.as_str() {
            "--check" => check = true,
            value => {
                return Err(format!("unknown synodal-engine-audit argument {value:?}").into());
            }
        }
    }
    let markdown = render(root)?;
    let output = root.join(OUTPUT);
    if check {
        if fs::read_to_string(&output)? != markdown {
            return Err(format!("stale {}; rerun cargo xtask synodal-engine-audit", OUTPUT).into());
        }
        println!("Synodal v0.10 productive morphology and lexicon audit: current");
    } else {
        fs::write(&output, markdown)?;
        println!("wrote {OUTPUT}");
    }
    Ok(())
}

fn render(root: &Path) -> Result<String, Box<dyn Error>> {
    let capabilities = read_matrix(root)?;
    let productive = capabilities
        .iter()
        .filter(|row| row.get(4).starts_with("productive"))
        .count();
    let unsupported = capabilities
        .iter()
        .filter(|row| row.get(4) == "unsupported")
        .count();
    let exact = capabilities
        .iter()
        .filter(|row| row.get(4).contains("exact"))
        .count();
    let irregular = capabilities
        .iter()
        .filter(|row| row.get(0) == "irregular")
        .count();

    let mut out = String::new();
    out.push_str("# Synodal v0.10 productive morphology and lexicon audit\n\n");
    out.push_str("## Headline result\n\n");
    out.push_str("The engine now productively covers seventeen reviewed noun contracts. The v0.10 slice adds complete `ѻтроча : ѻтрочат-`, `свекры : свекров-`, and ordinary `камень : камен-` contracts. It keeps the `каменїе` collective separate, retains unsafe `любовь` alternant generalization as an evidence blocker, and strengthens registered verb-system diagnostics without deriving systems from an infinitive.\n\n");
    out.push_str("A public provider layer now composes generated and application-owned lexicons without runtime I/O. Exact provider cells, caller irregular cells, and productive fallback share the existing kernel; duplicate identities fail closed. Ordered batch and provider-paradigm APIs retain every typed failure.\n\n");
    out.push_str("Corpus coverage is not the optimization target. The frozen corpus checkpoint remains a regression baseline only; no v0.10 rule, lexical upgrade, or accent pattern was selected from frequency or coverage movement.\n\n");
    out.push_str("## Public engine contract\n\n");
    out.push_str("`NounSpec`, `AdjectiveSpec`, and `VerbSpec` accept closed linguistic types, validated Unicode stems and principal parts, explicit provenance, optional irregular/defective cells, and an optional typed `AccentParadigm`. Fourth-declension nouns require the independently supplied extended stem; `NounNumberInventory` makes absent numbers explicit. `PresentPrincipalParts` and `VerbSpecBuilder::present_series` install the three independent present inputs atomically. Registry, provider, and explicit routes delegate to the same pure productive kernel after identity and override layers.\n\n");
    out.push_str("`VerbSystem` selects every represented finite, imperative, infinitive, l-participle, participial, supine, and verbal-noun inventory through one paradigm API. Paradigms retain every attempted cell. `LexemeProvider`, `StaticLexemeProvider`, `InMemoryLexemeProvider`, and `Lexicon` add deterministic composition and capability inspection. `BatchResult`, `ParadigmStatus`, row-level error codes, and `ErrorCode` expose successes and failures without parsing diagnostic prose.\n\n");
    out.push_str("## Capability summary\n\n");
    out.push_str(&format!(
        "The matrix contains {} reviewed system/subtype rows: {productive} productive rows, {exact} rows involving exact tables, {irregular} explicit irregular rows, and {unsupported} unsupported rows. Counts describe engine contracts, not corpus forms or tokens.\n\n",
        capabilities.len()
    ));
    out.push_str("The machine-readable source of truth is `data/synodal/engine_capabilities.tsv`. Every row records its target recension, valid and invalid inventory, required metadata, alternations, accent contract, source citation, golden/boundary example, implementation, test, and typed failure.\n\n");
    out.push_str("## Complete capability matrix\n\n");
    out.push_str("| Category | Subtype | Status | Stable rule | Valid inventory | Required metadata | Accent contract | Citation | Typed failure |\n");
    out.push_str("|---|---|---|---|---|---|---|---|---|\n");
    for row in &capabilities {
        out.push_str(&format!(
            "| {} | {} | {} | `{}` | {} | {} | {} | {} {} | `{}` |\n",
            md(row.get(0)),
            md(row.get(1)),
            md(row.get(4)),
            md(row.get(2)),
            md(row.get(5)),
            md(row.get(7)),
            md(row.get(9)),
            md(row.get(10)),
            md(row.get(11)),
            md(row.get(16)),
        ));
    }

    out.push_str("\n## New source-backed morphology\n\n");
    out.push_str("- `SYN-NOUN-I-HARD-VELAR-M-ALYPY-34` implements the reviewed г/к/х alternations at the exact §34 seams, with separate first and second palatalization behavior and boundary tests for all three velars.\n");
    out.push_str("- `SYN-NOUN-I-MIXED-M-ALYPY-33-34` implements the complete `мꙋжъ` mixed paradigm and its ordered nominative variants. Alypy §35 says `-(ь)ми` is lexical and unavailable to some nouns, so the class deliberately does not invent it.\n");
    out.push_str("- `SYN-NOUN-III-M-ALYPY-41` implements the complete `пꙋть` consonantal paradigm, including ordered vocative and genitive-plural variants. `NounNumberInventory` separately represents plural-only nouns such as `людїе`.\n");
    out.push_str("- `SYN-NOUN-IV-N-EN-ALYPY-42-43`, `SYN-NOUN-IV-N-ES-ALYPY-42-43`, and `SYN-NOUN-IV-F-ER-ALYPY-42-43` require explicit extended stems and implement the complete `имѧ : имен-`, `небо : небес-`, and `мати : матер-` tables, including reviewed wide-letter alternations and ordered ending variants.\n\n");
    out.push_str("- `SYN-NOUN-IV-N-AT-ALYPY-42-43` requires an independent `-ат-` stem and implements the complete `ѻтроча` table, including source-defined wide-letter and ending variants.\n");
    out.push_str("- `SYN-NOUN-IV-F-OV-ALYPY-42-44` implements the complete `свекры` ending family without treating cell-specific `любовь` alternants as class-wide.\n");
    out.push_str("- `SYN-NOUN-IV-M-EN-ALYPY-42-44` implements the reusable masculine `-ен-` family without leaking lexeme-specific variants. `SYN-NOUN-IV-M-EN-KAMEN-ALYPY-43` adds only the cited `камень` alternatives; the collective `каменїе` remains separate.\n\n");
    out.push_str("## Lexical providers and ordered batches\n\n");
    out.push_str("`StaticLexemeProvider` adapts the generated registry to the same `LexemeProvider` snapshot contract as application entries. `Lexicon::compose` sorts by stable ID, rejects duplicate IDs with `ProviderConflict`, and preserves homographic ambiguity. Supplied exact cells win before irregular cells and productive fallback. `Lexicon::batch`, provider noun paradigms, and provider `VerbSystem` paradigms retain order, variants, provenance, and one typed outcome per request.\n\n");
    out.push_str("## Reusable accent realization\n\n");
    out.push_str("`synodal-accent:mati-fixed-stem`, `synodal-accent:imya-mobile`, and `synodal-accent:nebo-mobile` encode the complete Alypy §43 tables as reusable rules. `мати` uses fixed first-stem-vowel stress. `имѧ` and `небо` use disjoint number-and-case scopes with stem/ending placement and acute/grave selection; `имѧ` also preserves initial psili before the accent mark. The implementation rejects missing and overlapping scopes, preserves exact-cell precedence, and retains `OrthographicMetadataRequired` when no rule applies.\n\n");
    out.push_str("## Irregular and defective behavior\n\n");
    out.push_str("Exact attested rows remain attestations. Normative cells in a declared irregular system are tagged `SynodalIrregularOverride`. In addition to `сынъ`, the upgraded `мꙋжъ` identity now demonstrates exact-first resolution with a productive mixed-declension background for uncovered cells. The reviewed `имѧ`, `небо`, and `мати` identities retain their exact normative tables while carrying productive classes and reusable accent metadata. Plural-only `людїе` adds a separately evidenced `NounNumberInventory` restriction so singular and dual requests remain typed invalid cells. Explicit specs can attach caller-specified overrides and retain historically absent or evidence-incomplete cells as distinct outcomes.\n\n");
    out.push_str("## Behavioral verification\n\n");
    out.push_str("The engine tests cover complete new noun paradigms, collective/ordinary plural separation, number restrictions, ordered variants, independent present edges and non-present stems, unified verb-system inventories, precise missing-formation diagnostics, provider conflicts and precedence, ordered batches, stable error codes, reusable accents, combining-mark order, and hostile Unicode. `data/synodal/linguistic_evaluation.tsv` adds eleven small source-linked contracts evaluated without frequency weighting.\n\n");
    out.push_str("The completion gate includes the package-specific and complete workspace suites, doctests, clippy with warnings denied, native no-default-feature builds, `wasm32-unknown-unknown` builds, byte-current generated registries and audit, package dry-runs, and a separate full-diff review.\n\n");
    out.push_str("The completion gate is:\n\n```text\ncargo fmt --all -- --check\ncargo clippy --workspace --all-targets --all-features -- -D warnings\ncargo test -p synodal-church-slavonic-core --all-features\ncargo test -p synodal-church-slavonic --all-features\ncargo test -p synodal-church-slavonic-dictionary --all-features\ncargo test -p synodal-church-slavonic-extractor --all-features\ncargo test --workspace --all-targets --all-features\ncargo test --workspace --doc\ncargo xtask synodal-engine-audit --check\ncargo xtask synodal-check\ncargo xtask check-all\n```\n\n");
    out.push_str("## Remaining source blockers\n\n");
    for row in capabilities
        .iter()
        .filter(|row| row.get(4) == "unsupported")
    {
        out.push_str(&format!(
            "- **{} / {}:** {} Failure: `{}`.\n",
            row.get(0),
            row.get(1),
            row.get(6),
            row.get(16)
        ));
    }
    out.push_str("\nSimple future, underspecified finite past, pronouns, and cardinal/collective numerals remain exact-table systems. The engine does not claim complete Church Slavonic support.\n\n");
    out.push_str("## Corpus regression policy\n\n");
    out.push_str("Corpus evaluation remains available only as a regression signal. The v0.10 implementation was selected and validated from complete target-recension grammatical tables, explicit API invariants, and independently reviewed lexical metadata; no frequency-ranked exact forms were added to simulate morphology.\n");
    Ok(out)
}

fn read_matrix(root: &Path) -> Result<Vec<Capability>, Box<dyn Error>> {
    let text = fs::read_to_string(root.join(MATRIX))?;
    let mut lines = text.lines();
    if lines.next() != Some(HEADER) {
        return Err(format!("{MATRIX} has an unexpected header").into());
    }
    let mut rows = Vec::new();
    for (offset, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields = line.split('\t').map(str::to_owned).collect::<Vec<_>>();
        if fields.len() != 17 {
            return Err(format!("{MATRIX}:{} has {} fields", offset + 2, fields.len()).into());
        }
        if fields[2].is_empty()
            || fields[3] != "synodal-russian"
            || fields[4].is_empty()
            || fields[5].is_empty()
            || fields[7].is_empty()
            || fields[10].is_empty()
            || fields[11].is_empty()
            || fields[15].is_empty()
            || fields[16].is_empty()
        {
            return Err(format!("{MATRIX}:{} has an incomplete contract", offset + 2).into());
        }
        rows.push(Capability { fields });
    }
    if rows.is_empty() {
        return Err(format!("{MATRIX} is empty").into());
    }
    Ok(rows)
}

fn md(value: &str) -> String {
    value.replace('|', "\\|").replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn committed_matrix_renders_deterministically() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let first = render(&root).expect("render");
        let second = render(&root).expect("render");
        assert_eq!(first, second);
        assert!(first.contains("SYN-NOUN-IV-N-AT-ALYPY-42-43"));
        assert!(first.contains("SYN-LEXICON-PROVIDER-V10"));
    }
}
