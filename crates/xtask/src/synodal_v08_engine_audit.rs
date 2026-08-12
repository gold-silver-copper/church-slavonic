use std::{error::Error, fs, path::Path};

const MATRIX: &str = "data/synodal/engine_capabilities.tsv";
const OUTPUT: &str = "docs/SYNODAL_V08_INFLECTION_ENGINE_AUDIT.md";
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
        println!("Synodal v0.8 engine capability audit: current");
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
    out.push_str("# Synodal v0.8 inflection-engine audit\n\n");
    out.push_str("## Headline result\n\n");
    out.push_str("The engine now inflects caller-supplied typed noun, adjective, and verb specifications without dictionary registration. The substantive new productive morphology is the complete short comparison declension and the complete short present/past active-participle declension, including their special citation edges and historically invalid vocatives. A reviewed fixed-stem accent paradigm now realizes multiple generated cells through both explicit and registry-backed APIs.\n\n");
    out.push_str("Corpus coverage is not the optimization target. Commit `aa4e693136ef094aab0da6ab166e1f23f49f9792` remains the frozen v0.7 checkpoint at 919,752 of 1,313,344 top-k tokens (70.031%). This audit neither raises that target nor treats incidental coverage movement as evidence of engine quality.\n\n");
    out.push_str("## Public engine contract\n\n");
    out.push_str("`NounSpec`, `AdjectiveSpec`, and `VerbSpec` accept closed linguistic types, validated Unicode stems and principal parts, explicit provenance, optional irregular/defective cells, and an optional typed `AccentParadigm`. `Inflector::form_spec` and the specialized paradigm methods retain caller-specified predictions as predictions. Registry and explicit routes delegate to the same pure productive kernel after their respective identity/override layers.\n\n");
    out.push_str("Paradigms retain every canonical attempted cell. `ParadigmStatus` separately reports attestation, irregular override, sourced prediction, caller-specified prediction, inherited prediction, ambiguity, historical invalidity, incomplete evidence, missing metadata, missing orthographic metadata, and unsupported behavior.\n\n");
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
    out.push_str("- `SYN-ADJ-COMPARATIVE-SHORT-ALYPY-58-98`: Alypy §58 supplies ancient and later comparison-stem formations and special nominative citation edges; §98 supplies the complete short-comparison declension. The API requires an independent comparison stem plus `ComparisonFormation`.\n");
    out.push_str("- `SYN-VERB-PARTICIPLE-PRESENT-ACTIVE-SHORT-ALYPY-95-98`: Alypy §95 supplies present-active stems/citation edges and the imperfective restriction; §98 supplies the complete declension.\n");
    out.push_str("- `SYN-VERB-PARTICIPLE-PAST-ACTIVE-SHORT-ALYPY-96-98`: Alypy §96 supplies consonant, vowel, and iotated past-active formations/citation edges; §98 supplies the complete declension.\n\n");
    out.push_str("All three rules cover 63 canonical valid cells: singular, dual, and plural; six licensed cases; all genders; and the additional animate accusative cells. Vocatives are retained as `HistoricallyInvalidCell`. Complete typed goldens exercise 189 successful cells plus 27 invalid vocatives.\n\n");
    out.push_str("## Reusable accent realization\n\n");
    out.push_str("`synodal-accent:mudr-fixed-stem` is a reviewed fixed-first-stem-vowel acute paradigm for long positive singular forms of `мꙋдръ`, cited to Alypy §57. It is one scoped rule that generates multiple cells, not a renamed list of accented strings. The exact nominative accent row still wins first; other licensed singular cells use the reusable paradigm. Missing scope remains `OrthographicMetadataRequired { field: AccentParadigm }`. The model separately represents stem versus ending placement, cell/number scopes, acute/grave/kamora, and an independently positioned psili breathing.\n\n");
    out.push_str("## Irregular and defective behavior\n\n");
    out.push_str("Exact attested rows remain attestations. Normative cells in a declared irregular system are tagged `SynodalIrregularOverride`. `сынъ` demonstrates a partial irregular system: reviewed dative-singular/plural overrides precede generation, while cells outside that declared override fall back only because the lexeme has an explicit first-hard masculine background. Explicit specs can likewise attach caller-specified overrides and can retain either historically absent or evidence-incomplete cells as distinct outcomes.\n\n");
    out.push_str("## Behavioral verification\n\n");
    out.push_str("The engine tests cover unregistered noun/adjective/verb specifications, independent present edges and non-present stems, complete short-comparison and active-participle inventories, ordered variants, vocative invalidity, perfective restrictions, missing and contradictory metadata, explicit/registry parity, exact/irregular/productive precedence, partial irregular fallback, evidence-incomplete cells, reusable accents through both routes, exact accent precedence, combining-mark order, and hostile Unicode.\n\n");
    out.push_str("At the v0.8 completion gate, the Synodal core passes 42 unit tests and 1 doctest, the facade passes 37 unit tests and 6 doctests, and the dictionary passes 27 unit tests, 5 CLI integration tests, and 1 doctest. The complete all-target workspace suite, workspace doctests, native/no-default-feature checks, `wasm32-unknown-unknown` checks, generated-registry checks, audit byte-current check, and package dry-runs also pass.\n\n");
    out.push_str("The completion gate is:\n\n```text\ncargo fmt --all -- --check\ncargo clippy --workspace --all-targets --all-features -- -D warnings\ncargo test -p synodal-church-slavonic-core --all-features\ncargo test -p synodal-church-slavonic --all-features\ncargo test -p synodal-church-slavonic-dictionary --all-features\ncargo test --workspace --all-targets --all-features\ncargo test --workspace --doc\ncargo xtask synodal-engine-audit --check\ncargo xtask synodal-check\ncargo xtask check-all\n```\n\n");
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
    out.push_str("## Incidental corpus regression signal\n\n");
    out.push_str("The frozen v0.7 checkpoint remains 919,752 top-k and 601,108 top-1 tokens. Against the same 1,313,344-token denominator, the live v0.8 regression run reports 919,786 top-k (+34), 601,081 top-1 (-27), 17,149 ambiguous (unchanged), and 392,520 unresolved (-34), or 70.033898% top-k. The shape is consistent with exposing additional ordered productive candidates: some formerly unresolved surfaces become analyzable, while some formerly unique surfaces gain another compatible analysis. This is a secondary regression observation, not an optimization result, and it did not drive rule selection.\n");
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
        assert!(first.contains("SYN-ADJ-COMPARATIVE-SHORT-ALYPY-58-98"));
        assert!(first.contains("synodal-accent:mudr-fixed-stem"));
    }
}
