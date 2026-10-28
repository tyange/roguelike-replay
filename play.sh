#!/usr/bin/env bash
# gitlogue 로 단계별 커밋을 재생한다.
#
#   ./play.sh              main 전체를 처음부터
#   ./play.sh 2-3          2-3 단계 커밋만 (stage-2-2..stage-2-3)
#   ./play.sh exp/refs     실험 브랜치 커밋만 (main 에서 갈라진 지점부터)
#   SPEED=30 ./play.sh 3-3 타이핑 속도 바꾸기 (기본 60)
#   PRINT=1 ./play.sh 2-3  실행하지 않고 gitlogue 명령만 출력
#   NO_CONTROLS=1 ./play.sh 2-3  컨트롤 바 없이 원본 gitlogue 로
#   ./play.sh list         단계와 커밋 목록
#
# ~/bin/rogue-replay 같은 심볼릭 링크로 불러도 저장소 폴더를 찾아간다
set -euo pipefail
cd "$(dirname "$(readlink -f "$0")")"

SPEED="${SPEED:-60}"
STAGES=(0 1-1 1-2 2-2 2-3 2-4 3-1 3-2 3-3 3-4 4-1 4-2 4-3 5-1 5-2 6)
COMMON=(--order asc --speed "$SPEED" --ignore play.sh --ignore README.md)

usage() {
  echo "사용법: $(basename "$0") [단계 | 브랜치 | list]" >&2
  echo "  단계: ${STAGES[*]}" >&2
  echo "  브랜치: $(git for-each-ref --format='%(refname:lstrip=-2)' 'refs/heads/exp/*' 'refs/remotes/origin/exp/*' | sort -u | tr '\n' ' ')" >&2
  exit 1
}

run() { # $1 = 커밋 범위
  # 컨트롤 바가 있는 포크(gitlogue-player)가 있으면 그걸 쓰고, 없으면 원본 gitlogue
  local cmd=(gitlogue --commit "$1" "${COMMON[@]}")
  if command -v gitlogue-player >/dev/null 2>&1 && [ -z "${NO_CONTROLS:-}" ]; then
    cmd=(gitlogue-player --commit "$1" "${COMMON[@]}" --controls)
  fi
  if [ -n "${PRINT:-}" ]; then
    echo "${cmd[*]}"
    return
  fi
  if ! command -v "${cmd[0]}" >/dev/null 2>&1; then
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

if [ "$1" = list ]; then
  prev=$ROOT
  for s in "${STAGES[@]}"; do
    echo "== $s"
    git log --reverse --format='   %s' "$prev..stage-$s"
    prev="stage-$s"
  done
  exit 0
fi
[ "$1" = help ] || [ "$1" = -h ] || [ "$1" = --help ] && usage

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

# 브랜치 이름이면 main 에서 갈라진 지점부터 브랜치 끝까지 (clone 직후라 원격에만 있어도 된다)
for ref in "refs/heads/$1" "refs/remotes/origin/$1"; do
  if git rev-parse --verify --quiet "$ref" >/dev/null; then
    base=$(git merge-base main "$ref")
    run "$base..$ref"
    exit 0
  fi
done

usage
