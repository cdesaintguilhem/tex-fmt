#!/usr/bin/env bash
echo "Getting performance metrics"
DIR="$(mktemp -d)"
cp -r ../tests/* "$DIR"

calc(){ awk "BEGIN { print ""$*"" }"; }

echo -n "Test files: $(find "$DIR"/*/* | wc -l | cut -w -f 2) files, "
echo -n "$(wc -l "$DIR"/source/* "$DIR"/target/* | grep "total" | cut -w -f 2) lines, "
echo "$(du -hs "$DIR" | cut -f 1)\n"


# tex-fmt
TEXFMTFILE="hyperfine-tex-fmt.csv"
echo "Building release binary"
cargo build --release
echo "Running benchmark"
hyperfine --warmup 10 \
    --min-runs 20 \
    --export-csv $TEXFMTFILE \
    --command-name "tex-fmt" \
    --prepare "cp -r ../tests/* $DIR" \
    "../target/release/tex-fmt $DIR/source/* $DIR/target/*"

# latexindent
LATEXINDENTFILE="hyperfine-latexindent.csv"
hyperfine --warmup 0 \
    --export-csv $LATEXINDENTFILE \
    --runs 1 \
    --command-name "latexindent" \
    --prepare "cp -r ../tests/* $DIR" \
    "latexindent $DIR/source/* $DIR/target/*"

# latexindent -m
LATEXINDENTMFILE="hyperfine-latexindent-m.csv"
hyperfine --warmup 0 \
    --export-csv $LATEXINDENTMFILE \
    --runs 1 \
    --command-name "latexindent -m" \
    --prepare "cp -r ../tests/* $DIR" \
    "latexindent -m $DIR/source/* $DIR/target/*"

# print results
TEXFMT=$(cat $TEXFMTFILE | tail -n 1 | cut -d "," -f 2)
echo "tex-fmt: ${TEXFMT}s"

LATEXINDENT=$(cat $LATEXINDENTFILE | tail -n 1 | cut -d "," -f 2)
LATEXINDENTTIMES=$(calc "$LATEXINDENT"/"$TEXFMT")
echo "latexindent: ${LATEXINDENT}s, x$LATEXINDENTTIMES"

LATEXINDENTM=$(cat $LATEXINDENTMFILE | tail -n 1 | cut -d "," -f 2)
LATEXINDENTMTIMES=$(calc "$LATEXINDENTM"/"$TEXFMT")
echo "latexindent -m: ${LATEXINDENTM}s, x$LATEXINDENTMTIMES"
