#!/usr/bin/env bash
set -euo pipefail

fixtures=("secrets" "sql-injection" "http-timeout" "server-xss" "clean")
expected=(1 1 1 1 0)

total_tp=0
total_fp=0
total_time=0

for i in "${!fixtures[@]}"; do
  name=${fixtures[$i]}
  exp=${expected[$i]}
  dir="fixtures/$name"

  rm -rf "$dir/.git" "$dir/index.json" "$dir/review_report.md"

  cargo run --quiet -p reviewlens -- --config "$dir/reviewer.toml" index --path "$dir" --output "$dir/index.json" >/dev/null

  git -C "$dir" init -q
  git -C "$dir" commit -qm init --allow-empty
  git -C "$dir" add .
  git -C "$dir" reset index.json reviewer.toml 2>/dev/null || true

  start=$(date +%s%N)
  cargo run --quiet -p reviewlens -- --config "$dir/reviewer.toml" check --path "$dir" --diff HEAD --output "$dir/review_report.md" >/dev/null
  end=$(date +%s%N)
  runtime=$(( (end-start)/1000000 ))
  total_time=$(( total_time + runtime ))

  count=$(rg -c '^\| ' "$dir/review_report.md" 2>/dev/null || echo 0)
  if [ "$count" -gt 0 ]; then
    findings=$((count - 1))
  else
    findings=0
  fi
  tp=$(( findings < exp ? findings : exp ))
  fp=$(( findings > exp ? findings - exp : 0 ))
  total_tp=$(( total_tp + tp ))
  total_fp=$(( total_fp + fp ))

  echo "$name: runtime=${runtime}ms findings=$findings expected=$exp"
done

if [ $((total_tp + total_fp)) -gt 0 ]; then
  precision=$(awk "BEGIN {printf \"%.2f\", $total_tp/($total_tp+$total_fp)}")
else
  precision="0.00"
fi

echo "Total runtime: ${total_time}ms"
echo "Precision: $precision"
