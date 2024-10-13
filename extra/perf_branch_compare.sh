#!/usr/bin/env bash
echo "Getting performance metrics"

BRANCH="pr-refactor-logic"
echo "Comparing branch ${BRANCH} with main"

DIR="$(mktemp -d)"
cp -r ../tests/* "$DIR"

calc(){ awk "BEGIN { print ""$*"" }"; }

echo -n "Test files: $(find "$DIR"/*/* | wc -l | cut -w -f 2) files, "
echo -n "$(wc -l "$DIR"/source/* "$DIR"/target/* | grep "total" | cut -w -f 2) lines, "
echo "$(du -hs "$DIR" | cut -f 1)\n"


# tex-fmt on main
TEXFMTFILE="hyperfine-tex-fmt.csv"
git checkout main
echo "Building release binary"
cargo build --release
echo "Running benchmark"
hyperfine --warmup 20 \
    --min-runs 200 \
    --export-csv $TEXFMTFILE \
    --command-name "tex-fmt-main" \
    --prepare "cp -r ../tests/* $DIR" \
    "../target/release/tex-fmt $DIR/source/* $DIR/target/*"

# tex-fmt on branch
TEXFMTFILEBRANCH="hyperfine-tex-fmt-branch.csv"
git checkout $BRANCH
echo "Building release binary"
cargo build --release
echo "Running benchmark"
hyperfine --warmup 20 \
    --min-runs 200 \
    --export-csv $TEXFMTFILEBRANCH \
    --command-name "tex-fmt-branch" \
    --prepare "cp -r ../tests/* $DIR" \
    "../target/release/tex-fmt $DIR/source/* $DIR/target/*"

# print results
TEXFMT=$(cat $TEXFMTFILE | tail -n 1 | cut -d "," -f 2)
echo "tex-fmt-main: ${TEXFMT}s"
TEXFMT=$(cat $TEXFMTFILEBRANCH | tail -n 1 | cut -d "," -f 2)
echo "tex-fmt-branch: ${TEXFMT}s"

# Return to perf-check branch
git checkout perf-check
