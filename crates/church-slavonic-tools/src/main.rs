//! `cargo xtask <command>`:
//!
//! - `eval` — the three numbers (held-out recall, Bible coverage, guesser
//!   accuracy);
//! - `build-treebank` / `check-treebank` — the Bible treebank.

use std::error::Error;


fn option(args: &mut Vec<String>, name: &str) -> Result<Option<String>, Box<dyn Error>> {
    if args.iter().filter(|a| a.as_str() == name).count() > 1 {
        return Err(format!("duplicate option {name}").into());
    }
    let Some(i) = args.iter().position(|a| a == name) else { return Ok(None); };
    let value = args.get(i + 1).cloned().ok_or_else(|| format!("{name} requires a value"))?;
    args.drain(i..i + 2);
    Ok(Some(value))
}
fn index_options(args: &mut Vec<String>) -> Result<Option<(String, String)>, Box<dyn Error>> {
    match (option(args, "--index")?, option(args, "--index-sha256")?) {
        (None, None) => Ok(None),
        (Some(path), Some(digest)) => Ok(Some((path, digest))),
        _ => Err("--index and caller-trusted --index-sha256 must be supplied together".into()),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // `--corpus ponomar[/<book>]` anywhere in the arguments selects the
    // corpus every treebank command runs over (4.1); the Bible otherwise
    let mut raw: Vec<String> = std::env::args().skip(1).collect();
    if let Some(i) = raw.iter().position(|a| a == "--corpus") {
        let spec = raw.get(i + 1).cloned().ok_or("--corpus <ponomar[/book]>")?;
        church_slavonic_tools::treebank::corpus::select(&spec)?;
        raw.drain(i..i + 2);
    }
    let mut args = raw.into_iter();
    match args.next().as_deref() {
        Some("corpus-observations") => {
            use std::io::Write;
            let kind = args.next().ok_or("corpus-observations <ud-train|ud-heldout|syntacticus>")?;
            if args.next().is_some() { return Err("unexpected corpus-observations argument".into()); }
            let root = church_slavonic_tools::workspace_root();
            let sources = root.join("references/downloads");
            let artifacts = root.join("target/sources");
            let corpus = match kind.as_str() {
                "syntacticus" => church_slavonic_tools::sources::ud::load_syntacticus(&sources, &artifacts)?,
                "ud-train" => church_slavonic_tools::sources::ud::load_ud_proiel_train(&sources, &artifacts)?,
                "ud-heldout" => church_slavonic_tools::sources::ud::load_ud_proiel_heldout(&sources, &artifacts)?,
                _ => return Err("corpus-observations <ud-train|ud-heldout|syntacticus>".into()),
            }.ok_or("requested corpus absent")?;
            let mut out = std::io::BufWriter::new(std::io::stdout().lock());
            corpus.write_observations(&mut out)?;
            out.flush()?;
            Ok(())
        }
        Some("eval-generation") => {
            let kind = args.next().ok_or("eval-generation <ud-heldout|syntacticus>")?;
            if args.next().is_some() { return Err("unexpected eval-generation argument".into()); }
            church_slavonic_tools::eval::generation::run(&kind)
        }
        Some("eval") => church_slavonic_tools::eval::run(args.collect()),
        Some("import") => church_slavonic_tools::import::run(args.collect()),
        Some("census") => church_slavonic_tools::census::run(args.collect()),
        Some("refit-stress") => {
            let args: Vec<String> = args.collect();
            let pos = match args.iter().position(|a| a == "--pos").and_then(|p| args.get(p + 1)).map(String::as_str) {
                Some("noun") => church_slavonic::Pos::Noun,
                Some("adj") => church_slavonic::Pos::Adjective,
                Some("verb") => church_slavonic::Pos::Verb,
                Some("pron") => church_slavonic::Pos::Pronoun,
                _ => return Err("refit-stress --pos <noun|adj|verb|pron> [--write]".into()),
            };
            church_slavonic_tools::import::refit::run(pos, args.iter().any(|a| a == "--write"))
        }
        Some("train-tagger") => church_slavonic_tools::tagger::train(&args.collect::<Vec<_>>()),
        Some("tagger-curve") => church_slavonic_tools::tagger::curve(),
        Some("tag-sequence") => {
            let mut args: Vec<String> = args.collect();
            let ocs = args.iter().position(|a| a == "--ocs").map(|i| args.remove(i)).is_some();
            let policy = if let Some(i) = args.iter().position(|a| a == "--match") {
                if i + 1 >= args.len() { return Err("--match requires a policy".into()); }
                let policy = church_slavonic_tools::analysis_document::parse_policy(&args[i+1])?;
                args.drain(i..=i+1); policy
            } else { church_slavonic::matching::MatchPolicy::LegacyOrthographic };
            if args.is_empty() { return Err("tag-sequence [--ocs] [--match policy] <token>…".into()); }
            let lexicon = if ocs { church_slavonic::Lexicon::ocs() } else { church_slavonic::Lexicon::synodal() };
            let predictions = church_slavonic_tools::tagger::deployment::predict(lexicon, &church_slavonic_tagger::Tagger::bundled(), &args, policy);
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                "pipeline":"raw-lookup-sequential-feature-tagger-v1",
                "matching_policy":church_slavonic_tools::analysis_document::policy_name(policy),
                "tokens":predictions.iter().map(church_slavonic_tools::tagger::deployment::prediction_json).collect::<Vec<_>>()
            }))?);
            Ok(())
        }
        Some("tagger-transfer") => church_slavonic_tools::tagger::transfer(&args.collect::<Vec<_>>()),
        Some("export") => church_slavonic_tools::treebank::export::run(),
        Some("titlo") => {
            // what the titlo index holds for each surface (3.3 debugging)
            let lexicon = church_slavonic::Lexicon::synodal();
            let index = church_slavonic_tools::treebank::lift::TitloIndex::build(lexicon);
            for word in args {
                match index.entries(&word) {
                    Some(v) => println!("{word}: {}", v.iter().map(|(p, id, c, a, full)| format!("{p}/{full} {id} {} alt {a}", c.name())).collect::<Vec<_>>().join(" | ")),
                    None => println!("{word}: no row abbreviates it"),
                }
            }
            Ok(())
        }
        Some("build-treebank") => church_slavonic_tools::treebank::runner::run(true),
        Some("check-treebank") => church_slavonic_tools::treebank::runner::run(false),
        Some("fix-hand-alts") => church_slavonic_tools::treebank::runner::fix_hand_alts(),
        Some("redraft-hand") => church_slavonic_tools::treebank::runner::redraft_hand(),
        Some("narrow-hand") => church_slavonic_tools::treebank::runner::narrow_hand(),
        Some("score-disambiguation") => church_slavonic_tools::treebank::runner::score_disambiguation(),
        Some("hand-draft") => {
            let args: Vec<String> = args.collect();
            let book: usize = args.first().and_then(|a| a.parse().ok()).ok_or("hand-draft <book index> <chapter>")?;
            let chapter: u32 = args.get(1).and_then(|a| a.parse().ok()).ok_or("hand-draft <book index> <chapter>")?;
            church_slavonic_tools::treebank::runner::hand_draft(book, chapter)
        }
        Some("filter-ud") => {
            let root = church_slavonic_tools::workspace_root();
            church_slavonic_tools::sources::ud::filter_train(&root.join("references/downloads"), &root.join("target/sources"), &root.join("data/intermediate/ud_proiel.jsonl"))
        }
        Some("check-linguistic-slice") => {
            use church_slavonic_tools::model_artifact;
            let args: Vec<String> = args.collect();
            if args.len() < 4 { return Err("check-linguistic-slice <model.json> <fixture.json> <lexeme-id> <orthography-id> [source-file…]".into()); }
            let sources: Vec<std::path::PathBuf> = args[4..].iter().map(Into::into).collect();
            let model = model_artifact::load_with_sources(std::path::Path::new(&args[0]), &sources)?;
            let fixture = serde_json::from_slice(&model_artifact::read_bounded(std::path::Path::new(&args[1]))?)?;
            let report = model_artifact::check_fixture(&model, fixture,
                &church_slavonic::morphology::LexemeId(args[2].clone()),
                &church_slavonic::morphology::OrthographyId(args[3].clone()))?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            if !report.failures.is_empty() { return Err("linguistic slice failed".into()); }
            Ok(())
        }
        Some("analyze-witness") => {
            let mut args: Vec<String> = args.collect();
            let compiled = index_options(&mut args)?;
            let policy = option(&mut args, "--match")?.map(|s| church_slavonic_tools::analysis_document::parse_policy(&s)).transpose()?.unwrap_or(church_slavonic::matching::MatchPolicy::Exact);
            if args.len() < 3 { return Err("analyze-witness <model.json> <orthography-id> <witness-id> [source-file…] [--match <policy>] [--index <file> --index-sha256 <trusted-digest>]".into()); }
            let sources: Vec<std::path::PathBuf> = args[3..].iter().map(Into::into).collect();
            // Source validation precedes cache installation; a compiled index
            // cannot replace or bypass the original registered witness bytes.
            let mut model = church_slavonic_tools::model_artifact::load_with_sources(std::path::Path::new(&args[0]), &sources)?;
            let orthography = church_slavonic::morphology::OrthographyId(args[1].clone());
            if let Some((path, digest)) = compiled {
                church_slavonic_tools::compiled_index::load_into(&mut model, std::path::Path::new(&path), &digest, &orthography, policy)?;
            }
            let document = church_slavonic::document::AnalysisDocument::analyze_registered_witness(
                &model, church_slavonic::morphology::observations::WitnessId(args[2].clone()), orthography, policy)?;
            println!("{}", church_slavonic_tools::analysis_document::to_json(&document)?);
            Ok(())
        }
        Some("context-trace") => {
            let mut texts: Vec<_> = args.collect();
            let ocs = texts.first().is_some_and(|s| s == "--ocs");
            if ocs { texts.remove(0); }
            if texts.len() != 1 { return Err("context-trace [--ocs] <text>".into()); }
            let lexicon = if ocs { church_slavonic::Lexicon::ocs() } else { church_slavonic::Lexicon::synodal() };
            let record = church_slavonic_tools::context_trace::evaluate(lexicon, &texts[0])?;
            println!("{}", church_slavonic_tools::context_trace::to_json(&record)?);
            Ok(())
        }
        Some("migrate-noun-records") => {
            let args: Vec<String> = args.collect();
            if args.len()!=4 { return Err("migrate-noun-records <legacy.tsv> <model.json> <mapping.json> <output.json>".into()); }
            let report=church_slavonic_tools::migration::migrate(std::path::Path::new(&args[0]),std::path::Path::new(&args[1]),std::path::Path::new(&args[2]),std::path::Path::new(&args[3]))?;
            println!("{}",serde_json::to_string_pretty(&report)?);
            Ok(())
        }
        Some("compile-index") => {
            let args: Vec<String> = args.collect();
            if args.len() < 4 { return Err("compile-index <model.json> <orthography> <matching-policy> <output.idx> [source-file…]".into()); }
            let policy = church_slavonic_tools::analysis_document::parse_policy(&args[2])?;
            let sources: Vec<std::path::PathBuf> = args[4..].iter().map(Into::into).collect();
            let receipt = church_slavonic_tools::compiled_index::compile(std::path::Path::new(&args[0]),
                church_slavonic::morphology::OrthographyId(args[1].clone()), policy, std::path::Path::new(&args[3]), &sources)?;
            println!("{}", serde_json::to_string_pretty(&receipt)?);
            Ok(())
        }
        Some("analyze") => {
            let mut args: Vec<String> = args.collect();
            let ocs = args.iter().position(|a| a == "--ocs").map(|i| args.remove(i)).is_some();
            let policy = if let Some(i) = args.iter().position(|a| a == "--match") {
                let policy = church_slavonic_tools::analysis_document::parse_policy(args.get(i + 1).ok_or("--match requires a policy")?)?;
                args.drain(i..i + 2);
                policy
            } else { church_slavonic::matching::MatchPolicy::Exact };
            let model_path = option(&mut args, "--model")?;
            let orthography = option(&mut args, "--orthography")?;
            let compiled = index_options(&mut args)?;
            if compiled.is_some() && model_path.is_none() { return Err("--index requires --model".into()); }
            if args.is_empty() { return Err("analyze requires a text argument (which may be an empty string)".into()); }
            if let Some(path) = model_path {
                if ocs { return Err("--ocs selects the legacy lexicon and cannot be combined with --model".into()); }
                let orthography = orthography.ok_or("--model requires --orthography <id>")?;
                let mut model = church_slavonic_tools::model_artifact::load(std::path::Path::new(&path))?;
                if let Some((index, digest)) = &compiled {
                    church_slavonic_tools::compiled_index::load_into(&mut model, std::path::Path::new(index), digest,
                        &church_slavonic::morphology::OrthographyId(orthography.clone()), policy)?;
                }
                for text in args {
                    let document = church_slavonic::document::AnalysisDocument::analyze_model(&model,
                        church_slavonic::witness::Witness::new(text, None),
                        church_slavonic::morphology::OrthographyId(orthography.clone()), policy)?;
                    println!("{}", church_slavonic_tools::analysis_document::to_json(&document)?);
                }
                return Ok(());
            }
            if orthography.is_some() { return Err("--orthography requires --model".into()); }
            let lexicon = if ocs { church_slavonic::Lexicon::ocs() } else { church_slavonic::Lexicon::synodal() };
            // Each argument is a separate witness, preserving its original bytes.
            for text in args {
                let document = church_slavonic::document::AnalysisDocument::analyze(
                    lexicon, church_slavonic::witness::Witness::new(text, None), policy);
                println!("{}", church_slavonic_tools::analysis_document::to_json(&document)?);
            }
            Ok(())
        }
        Some("-h") | Some("--help") | None => {
            eprintln!("cargo xtask <migrate-noun-records <legacy.tsv> <model.json> <mapping.json> <output.json> | compile-index <model.json> <orthography> <policy> <output.idx> [source-file…] | context-trace [--ocs] <text> | eval-generation <ud-heldout|syntacticus> | corpus-observations <ud-train|ud-heldout|syntacticus> | tag-sequence [--ocs] [--match policy] <token>… | tagger-curve | analyze-witness <model.json> <orthography-id> <witness-id> [source-file…] [--match <policy>] [--index <file> --index-sha256 <trusted-digest>] | check-linguistic-slice <model.json> <fixture.json> <lexeme-id> <orthography-id> [source-file…] | eval [--guess verbs [--ocs]] | census <stems --pos <pos> [--ocs] | verb-cells --ocs | closed | clitics | homonymy | stress> | import <source> --pos <pos> [--write] | build-treebank | check-treebank | fix-hand-alts | narrow-hand | score-disambiguation | hand-draft <book> <chapter> | analyze [--ocs | --model <file> --orthography <id> [--index <file> --index-sha256 <trusted-digest>]] [--match <exact|unicode-equivalent|case-insensitive|accent-insensitive|legacy-orthographic>] <text>…>");
            Ok(())
        }
        Some(other) => Err(format!("unknown xtask command: {other}").into()),
    }
}
