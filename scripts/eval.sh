#!/usr/bin/env bash
set -euo pipefail

fixtures=("secrets" "sql-injection" "http-timeout" "server-xss" "clean")
expected=(1 1 1 1 0)

total_tp=0
total_fp=0
runtimes=()
memories=()

for i in "${!fixtures[@]}"; do
  name=${fixtures[$i]}
  exp=${expected[$i]}
  dir="fixtures/$name"

  rm -rf "$dir/.git" "$dir/index.json" "$dir/review_report.md"

  /usr/bin/time -f "%M" -o /tmp/mem.txt \
    cargo run --quiet -p reviewlens -- --config "$dir/reviewlens.toml" index \
    --path "$dir" --output "$dir/index.json" >/dev/null 2>&1

  git -C "$dir" init -q
  git -C "$dir" commit -qm init --allow-empty
  git -C "$dir" add .
  git -C "$dir" reset index.json reviewlens.toml 2>/dev/null || true

  start=$(date +%s%N)
  /usr/bin/time -f "%M" -o /tmp/mem.txt \
    cargo run --quiet -p reviewlens -- --config "$dir/reviewlens.toml" check \
    --path "$dir" --diff HEAD --output "$dir/review_report.md" >/dev/null 2>&1 || true
  end=$(date +%s%N)
  runtime=$(( (end-start)/1000000 ))
  memory=$(tail -n 1 /tmp/mem.txt)
  runtimes+=("$runtime")
  memories+=("$memory")

  security_section=$(awk '/^## .*Security Findings/{flag=1;next}/^## /{flag=0}flag' "$dir/review_report.md")
  if echo "$security_section" | rg -q 'No issues found'; then
    findings=0
  else
    count=$(echo "$security_section" | rg -c '^\| ' || true)
    findings=$(( count > 0 ? count - 1 : 0 ))
  fi

  tp=$(( findings < exp ? findings : exp ))
  fp=$(( findings > exp ? findings - exp : 0 ))
  total_tp=$(( total_tp + tp ))
  total_fp=$(( total_fp + fp ))

  echo "$name: runtime=${runtime}ms memory=${memory}KB findings=$findings expected=$exp"
done

sorted_runtime=($(printf '%s\n' "${runtimes[@]}" | sort -n))
sorted_memory=($(printf '%s\n' "${memories[@]}" | sort -n))
count=${#sorted_runtime[@]}
mid=$(((count-1)/2))
p95_idx=$((95*(count-1)/100))
p50_runtime=${sorted_runtime[$mid]}
p95_runtime=${sorted_runtime[$p95_idx]}
p50_memory=${sorted_memory[$mid]}
p95_memory=${sorted_memory[$p95_idx]}

if [ $((total_tp + total_fp)) -gt 0 ]; then
  precision=$(awk "BEGIN {printf \"%.2f\", $total_tp/($total_tp+$total_fp)}")
  fp_rate=$(awk "BEGIN {printf \"%.2f\", $total_fp/($total_tp+$total_fp)}")
else
  precision="0.00"
  fp_rate="0.00"
fi

if [ -n "${GITHUB_STEP_SUMMARY:-}" ]; then
  {
    echo "### Evaluation Summary"
    echo ""
    echo "| Metric | Value |"
    echo "| --- | --- |"
    echo "| Precision | $precision |"
    echo "| FP Rate | $fp_rate |"
    echo "| Runtime P50 (ms) | $p50_runtime |"
    echo "| Runtime P95 (ms) | $p95_runtime |"
    echo "| Memory P50 (KB) | $p50_memory |"
    echo "| Memory P95 (KB) | $p95_memory |"
  } >> "$GITHUB_STEP_SUMMARY"
fi

echo "Precision: $precision"
echo "FP Rate: $fp_rate"
echo "Runtime P50: ${p50_runtime}ms"
echo "Runtime P95: ${p95_runtime}ms"
echo "Memory P50: ${p50_memory}KB"
echo "Memory P95: ${p95_memory}KB"

