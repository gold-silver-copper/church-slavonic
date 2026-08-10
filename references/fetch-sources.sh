#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
DOWNLOAD_DIR="$SCRIPT_DIR/downloads"
TEMP_DIR="$SCRIPT_DIR/.tmp"

mkdir -p "$DOWNLOAD_DIR" "$TEMP_DIR"

fetch() {
    source_url=$1
    destination=$2
    mkdir -p "$(dirname -- "$destination")"
    if [[ -s "$destination" ]]; then
        printf 'present  %s\n' "${destination#"$SCRIPT_DIR"/}"
        return
    fi

    partial="$destination.partial"
    printf 'fetching %s\n' "${destination#"$SCRIPT_DIR"/}"
    curl --fail --silent --show-error --location --retry 4 --retry-delay 2 \
        --continue-at - \
        --user-agent 'church-slavonic-source-fetcher/1.0' \
        --output "$partial" "$source_url"
    mv "$partial" "$destination"
}

fetch_fup() {
    source_url=$1
    destination=$2
    mkdir -p "$(dirname -- "$destination")"
    if [[ -s "$destination" ]]; then
        printf 'present  %s\n' "${destination#"$SCRIPT_DIR"/}"
        return
    fi

    partial="$destination.partial"
    printf 'fetching %s\n' "${destination#"$SCRIPT_DIR"/}"
    # media.fupress.com serves these CC BY artifacts only to an ordinary
    # browser user agent and requires the public catalogue as the referrer.
    curl --http1.1 --fail --silent --show-error --location --retry 4 --retry-delay 2 \
        --continue-at - \
        --user-agent 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36' \
        --referer 'https://books.fupress.com/catalogue/old-church-slavic/8465' \
        --header 'Accept: application/xml,text/xml,application/pdf,application/xhtml+xml,text/html;q=0.9,*/*;q=0.8' \
        --output "$partial" "$source_url"
    mv "$partial" "$destination"
}

fetch_github_archive() {
    owner=$1
    repository=$2
    revision=$3
    source_id=$4
    fetch \
        "https://codeload.github.com/$owner/$repository/tar.gz/$revision" \
        "$DOWNLOAD_DIR/$source_id/$repository-$revision.tar.gz"
}

fetch_ccmh() {
    ccmh_dir="$DOWNLOAD_DIR/ccmh-2021-04-23"
    base_url='https://www.kielipankki.fi/download/ccmh-src/www'
    mkdir -p "$ccmh_dir"

    ccmh_files=(
        index.html
        assemanianus.html assemanianus.txt assemanianus.xml
        marianus.html marianus.txt marianus.xml
        suprasliensis.html suprasliensis.txt
        zographensis.html zographensis.txt zographensis.xml
        zogr-b.txt zogr_glag_cyr.xls
        savvina.html savvina.txt savvina.xml
        vita_constantini.html vita_constantini.txt
        vita_methodii.html vita_methodii.txt
    )
    for ccmh_file in "${ccmh_files[@]}"; do
        fetch "$base_url/$ccmh_file" "$ccmh_dir/$ccmh_file"
    done
}

fetch_wikisource_bible() {
    source_dir="$DOWNLOAD_DIR/wikisource-church-slavonic-bible"
    api_url='https://wikisource.org/w/api.php'
    root_title='Бі́блїа'
    mkdir -p "$source_dir"
    if [[ -s "$source_dir/export.xml" && -s "$source_dir/titles.txt" ]]; then
        printf 'present  %s\n' "${source_dir#"$SCRIPT_DIR"/}/export.xml"
        return
    fi

    titles_json="$TEMP_DIR/wikisource-bible-titles.json"
    curl --fail --silent --show-error --get --retry 8 --retry-all-errors \
        --retry-delay 3 "$api_url" \
        --data-urlencode 'action=query' \
        --data-urlencode 'format=json' \
        --data-urlencode 'formatversion=2' \
        --data-urlencode 'prop=links' \
        --data-urlencode "titles=$root_title" \
        --data-urlencode 'plnamespace=0' \
        --data-urlencode 'pllimit=max' \
        --output "$titles_json"

    titles_file="$TEMP_DIR/wikisource-bible-titles.txt"
    jq -r --arg root "$root_title" \
        '[ $root, (.query.pages[].links[]?.title) ] | unique[]' \
        "$titles_json" > "$titles_file"

    export_partial="$source_dir/export.xml.partial"
    curl --fail --silent --show-error --location --retry 8 --retry-all-errors \
        --retry-delay 3 --request POST \
        'https://wikisource.org/wiki/Special:Export' \
        --data-urlencode "pages@$titles_file" \
        --data 'curonly=1' \
        --data 'action=submit' \
        --output "$export_partial"
    mv "$export_partial" "$source_dir/export.xml"
    cp "$titles_file" "$source_dir/titles.txt"
}

fetch_alypy_grammar() {
    grammar_dir="$DOWNLOAD_DIR/alypy-grammar"
    grammar_base_url='https://www.ponomar.net/files/gama2'
    grammar_toc="$grammar_dir/toc.html"

    fetch "$grammar_base_url/toc.html" "$grammar_toc"
    while IFS= read -r grammar_page; do
        fetch "$grammar_base_url/$grammar_page" "$grammar_dir/$grammar_page"
    done < <(
        sed -n 's/.*href="\([^"]*\.htm\)".*/\1/p' "$grammar_toc" \
            | LC_ALL=C sort -u
    )
}

# OCS lexical and grammatical sources.
fetch \
    'https://kaikki.org/dictionary/Old%20Church%20Slavonic/kaikki.org-dictionary-OldChurchSlavonic.jsonl' \
    "$DOWNLOAD_DIR/english-wiktionary-ocs/kaikki.org-dictionary-OldChurchSlavonic.jsonl"
fetch \
    'https://dumps.wikimedia.org/enwiktionary/20260801/enwiktionary-20260801-pages-articles.xml.bz2' \
    "$DOWNLOAD_DIR/english-wiktionary-ocs-lineage/enwiktionary-20260801-pages-articles.xml.bz2"
fetch \
    'https://dumps.wikimedia.org/enwiktionary/20260801/enwiktionary-20260801-sha1sums.txt' \
    "$DOWNLOAD_DIR/english-wiktionary-ocs-lineage/enwiktionary-20260801-sha1sums.txt"
fetch \
    'https://dumps.wikimedia.org/enwiktionary/20260801/dumpstatus.json' \
    "$DOWNLOAD_DIR/english-wiktionary-ocs-lineage/dumpstatus.json"
fetch \
    'https://kaikki.org/dictionary/raw-wiktextract-data.jsonl.gz' \
    "$DOWNLOAD_DIR/english-wiktionary-ocs-lineage/raw-wiktextract-data.jsonl.gz"
fetch \
    'https://kaikki.org/dictionary/wiktionary-modules.tar.gz' \
    "$DOWNLOAD_DIR/english-wiktionary-ocs-lineage/wiktionary-modules.tar.gz"
fetch \
    'https://kaikki.org/dictionary/wiktionary-templates.tar.gz' \
    "$DOWNLOAD_DIR/english-wiktionary-ocs-lineage/wiktionary-templates.tar.gz"
fetch \
    'https://integral.github.io/osd/data/osd.zip' \
    "$DOWNLOAD_DIR/polivanova-osd/osd.zip"
fetch_fup \
    'https://media.fupress.com/files/xml/50/8465/37708' \
    "$DOWNLOAD_DIR/polivanova-fup/old-church-slavic-979-12-215-0105-6.xml"
fetch_fup \
    'https://media.fupress.com/files/pdf/24/8465/37707' \
    "$DOWNLOAD_DIR/polivanova-fup/old-church-slavic-979-12-215-0104-9.pdf"

# Attested OCS corpora. These are local/evaluation-only where noted in SOURCES.toml.
fetch_github_archive \
    UniversalDependencies UD_Old_Church_Slavonic-PROIEL \
    64eddf87abfaa51e7f5acf0bef1bebcdaca1559f ud-ocs-proiel-r2.18
fetch_github_archive \
    syntacticus syntacticus-treebank-data \
    525cee4fb40590d7d514376c11acaed1bdd91c15 syntacticus-20230428
fetch_ccmh
fetch_github_archive \
    MariaCassese DIACU \
    d4b00baa0b63b9ed4c60eb998670986a072294a0 diacu-1.0

# Synodal Russian Church Slavonic and comparative Church Slavonic sources.
fetch \
    'https://www.unicode.org/notes/tn41/tn41-1.pdf' \
    "$DOWNLOAD_DIR/unicode-tn41/tn41-1.pdf"
fetch \
    'https://www.unicode.org/notes/tn41/' \
    "$DOWNLOAD_DIR/unicode-tn41/index.html"
fetch_github_archive \
    typiconman ponomar \
    0af645f438856f45c22026912d2e4a9ce495e531 ponomar-elizabeth-bible
fetch \
    'https://www.crosswire.org/ftpmirror/pub/sword/packages/rawzip/CSlElizabeth.zip' \
    "$DOWNLOAD_DIR/crosswire-csl-elizabeth/CSlElizabeth-1.5.2.zip"
fetch_wikisource_bible
fetch \
    'https://www.ponomar.net/maktabah/index.html' \
    "$DOWNLOAD_DIR/ponomar-library/catalog.html"
fetch \
    'https://www.ponomar.net/legal.html' \
    "$DOWNLOAD_DIR/ponomar-library/legal.html"
fetch \
    'https://www.ponomar.net/files/wordlist.tsv' \
    "$DOWNLOAD_DIR/ponomar-modern-corpus/wordlist.tsv"
fetch \
    'https://www.ponomar.net/files/dictout.xls' \
    "$DOWNLOAD_DIR/ponomar-modern-corpus/dictout.xls"
fetch \
    'https://www.ponomar.net/files/cubooks.zip' \
    "$DOWNLOAD_DIR/ponomar-modern-corpus/cubooks.zip"
fetch_alypy_grammar
fetch \
    'https://upload.wikimedia.org/wikipedia/commons/1/14/%D0%9F%D0%BE%D0%BB%D0%BD%D1%8B%D0%B9_%D1%86%D0%B5%D1%80%D0%BA%D0%BE%D0%B2%D0%BD%D0%BE%D1%81%D0%BB%D0%B0%D0%B2%D1%8F%D0%BD%D1%81%D0%BA%D0%B8%D0%B9_%D1%81%D0%BB%D0%BE%D0%B2%D0%B0%D1%80%D1%8C_%28%D0%9F%D1%80%D0%BE%D1%82%D0%BE%D0%B8%D0%B5%D1%80%D0%B5%D0%B9_%D0%93.%D0%94%D1%8C%D1%8F%D1%87%D0%B5%D0%BD%D0%BA%D0%BE%29.djvu' \
    "$DOWNLOAD_DIR/dyachenko-1900/dyachenko-complete-church-slavonic-dictionary.djvu"

find "$DOWNLOAD_DIR" -type f -print0 \
    | LC_ALL=C sort -z \
    | xargs -0 shasum -a 256 \
    | sed "s|  $DOWNLOAD_DIR/|  downloads/|" \
    > "$SCRIPT_DIR/SHA256SUMS"

printf '\nFetched %s files (%s).\n' \
    "$(find "$DOWNLOAD_DIR" -type f | wc -l | tr -d ' ')" \
    "$(du -sh "$DOWNLOAD_DIR" | awk '{print $1}')"
