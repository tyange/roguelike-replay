#!/usr/bin/env bash
# gitlogue 로 단계별 커밋을 재생한다.
#
#   ./play.sh              main 전체를 처음부터
#   ./play.sh 2-3          2-3 단계 커밋만 (stage-2-2..stage-2-3)
#   ./play.sh exp/refs     실험 브랜치 커밋만 (main 에서 갈라진 지점부터)
#   SPEED=30 ./play.sh 3-3 타이핑 속도 바꾸기 (기본 60)
#   PRINT=1 ./play.sh 2-3  실행하지 않고 gitlogue 명령만 출력
set -euo pipefail
cd "$(dirname "$0")"

SPEED="${SPEED:-60}"
STAGES=(0 1-1 1-2 2-2 2-3 2-4 3-1 3-2 3-3 3-4 4-1 4-2 4-3 5-1 5-2 6)
COMMON=(--order asc --speed "$SPEED" --ignore play.sh --ignore README.md)

usage() {
  echo "사용법: ./play.sh [단계 | 브랜치]" >&2
  echo "  단계: ${STAGES[*]}" >&2
  echo "  브랜치: $(git for-each-ref --format='%(refname:short)' 'refs/heads/exp/*' | tr '\n' ' ')" >&2
  exit 1
}

run() { # $1 = 커밋 범위
  local cmd=(gitlogue --commit "$1" "${COMMON[@]}")
  if [ -n "${PRINT:-}" ]; then
    echo "${cmd[*]}"
    return
  fi
  if ! command -v gitlogue >/dev/null 2>&1; then
    echo "gitlogue 가 설치되어 있지 않습니다. 실행할 명령:" >&2
    echo "  ${cmd[*]}" >&2
    exit 1
  fi
  exec "${cmd[@]}"
}

# 루트 커밋(cargo new 결과물)은 범위의 시작점이라 재생에서 빠진다. cargo new 는 직접 실행한다
ROOT=$(git rev-list --max-parents=0 main)

if [ $# -eq 0 ]; then
  run "$ROOT..main"
  exit 0
fi
[ $# -eq 1 ] || usage

arg=${1#stage-}

# 단계 이름이면 이전 단계 태그부터 이 단계 태그까지
for i in "${!STAGES[@]}"; do
  if [ "${STAGES[$i]}" = "$arg" ]; then
    if [ "$i" -eq 0 ]; then
      from=$ROOT
    else
      from="stage-${STAGES[$((i - 1))]}"
    fi
    run "$from..stage-$arg"
    exit 0
  fi
done

# 브랜치 이름이면 main 에서 갈라진 지점부터 브랜치 끝까지
if git rev-parse --verify --quiet "refs/heads/$1" >/dev/null; then
  base=$(git merge-base main "$1")
  run "$base..$1"
  exit 0
fi

usage
