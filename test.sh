#!/bin/bash

# 테스트 실행 횟수
num_runs=10

# 테스트 항목 배열
tests=("model::task::tests::test_list_ok" "model::task::tests::test_create_ok" "model::task::tests::test_get_err_not_found" "model::task::tests::test_delete_err_not_found" "model::task::tests::test_update_ok")

# 성공/실패 카운터 배열 초기화
declare -A success_counts
declare -A fail_counts

for test in "${tests[@]}"; do
  key="${test//:/_}"
  success_counts["$key"]=0
  fail_counts["$key"]=0
done

# 각 테스트 항목에 대해 여러 번 실행
for test in "${tests[@]}"; do
  key="${test//:/_}"
  for (( i=1; i<=$num_runs; i++ )); do
    echo "Running $test - Run $i of $num_runs"
    if cargo test "$test"; then
      ((success_counts["$key"]++))
    else
      ((fail_counts["$key"]++))
    fi
  done
done

# 각 테스트 항목에 대한 통계 출력
for test in "${tests[@]}"; do
  key="${test//:/_}"
  echo "Test: $test"
  echo "  Successes: ${success_counts["$key"]}"
  echo "  Failures: ${fail_counts["$key"]}"
done
