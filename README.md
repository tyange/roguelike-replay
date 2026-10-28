# roguelike-replay

「로그라이크 직접 만들기 — 서술형 가이드 (Macroquad)」를 단계별 완성 코드로 쌓아 둔 저장소입니다.
커밋 하나가 파일 하나(또는 의미 단위 하나)이고, 각 단계의 마지막 커밋에 태그가 붙어 있습니다.
gitlogue 로 커밋 diff 를 타이핑 애니메이션으로 재생하면서 따라 치는 용도입니다.

- 구조: 루트에 `.gitignore`, 옆에 나란히 놓인 독립 패키지 `rogue-core/`, `rogue/`, (5-2 부터) `rogue-sim/`, (4-2 부터) 데이터 `data/defs.ron`
- 기준: Rust 1.97+, edition 2024, macroquad 0.4.16, serde 1, ron 0.12, rayon 1.12
- 모든 단계의 코드는 완성본입니다 (`TODO` / `todo!()` 없음). 일부러 실패하는 코드는 `exp/*` 브랜치에만 있습니다.

## 사용법

### 1. 준비 (한 번만)

```bash
brew install gitlogue
git clone git@github-personal:tyange/roguelike-replay.git
```

이 저장소는 **보기 전용**입니다. 따라 치는 코드는 가이드대로 별도 `roguelike` 폴더에 직접 만듭니다.

```bash
mkdir roguelike && cd roguelike
git init
cargo new --lib rogue-core
cargo new rogue
```

`cargo new` 결과물(루트 커밋)은 재생되지 않으니, 위 명령은 직접 실행하고 시작합니다.

### 2. 터미널 두 개 띄우기

| 왼쪽: 재생 | 오른쪽: 따라 치기 |
| --- | --- |
| `cd roguelike-replay && ./play.sh 0` | `cd roguelike` 에서 에디터로 입력 |

### 3. 한 단계 따라 치기

1. Craft 가이드에서 그 단계의 설명(만들 것, 설계 설명)을 먼저 읽습니다.
2. `./play.sh <단계>` 로 재생합니다. 예: `./play.sh 1-1`
3. 화면 위의 커밋 메시지(`[1-1] map.rs — 타일과 맵`)가 **지금 어느 파일을 치는지** 알려 줍니다. 커밋 하나 = 파일 하나입니다.
4. 따라가기 버거우면 `Space` 로 멈추고, `h` 로 몇 줄 되감습니다. 지난 파일을 다시 보려면 `p`.
5. 단계가 끝나면 내 폴더에서 확인합니다.
   ```bash
   (cd rogue-core && cargo test && cargo clippy)
   (cd rogue && cargo run)
   ```
6. 통과하면 내 저장소에 커밋하고 다음 단계로 갑니다.

속도는 `SPEED=100 ./play.sh 2-3` 처럼 바꿉니다 (글자당 ms, 클수록 느림, 기본 60).

### 4. 막혔을 때: 내 코드와 비교

재생을 멈추고 이 저장소의 해당 단계 완성본과 내 파일을 직접 비교합니다.

```bash
# 2-3 단계 완성 시점의 파일 보기
git -C roguelike-replay show stage-2-3:rogue-core/src/store.rs

# 내 파일과 diff
git -C roguelike-replay show stage-2-3:rogue-core/src/store.rs | diff - roguelike/rogue-core/src/store.rs

# 그 단계 완성본을 통째로 꺼내서 빌드해 보기
git -C roguelike-replay worktree add ../replay-2-3 stage-2-3
```

### 5. 실험 단계 (2-1, 3-2 구현 B)

가이드에서 실패용 브랜치를 만들라고 하는 단계는 내 저장소에서도 브랜치를 만들고 따라 칩니다.

```bash
./play.sh exp/refs     # 2-1: 1-2 를 끝낸 뒤 (내 저장소: git switch -c exp/refs)
./play.sh exp/dyn-ai   # 3-2 구현 B: 3-2 를 끝낸 뒤
```

끝나면 내 저장소는 main 으로 돌아오고 (`git switch main`), 다음 단계는 다시 `./play.sh 2-2` 부터 이어집니다.

### 주의

- 0, 1-1, 1-2 는 가이드 원문 그대로입니다. **2-2 부터는 가이드 뼈대 + 「로그라이크 정답 노트」 를 합치고, 두 문서에 없는 부분은 추론해서 채운 한 가지 완성본**입니다 (테스트와 clippy 는 통과). 내 코드가 모양이 달라도 가이드의 확인 기준을 통과하면 됩니다.
- 원문과 달리 고친 곳: 2-2 `draw_glyph` 의 `&ch.to_string()` → `ch.to_string()` (clippy), 3-3 테스트 두 개에 고블린 배치 추가와 `chaser_approaches` 위치 조정, 4-2 데이터 경로 `../data/defs.ron`, 4-3 `Defs` 에 `Default` derive.

## 단계와 태그

| 태그 | 단계 | 커밋 수 | 확인 |
| --- | --- | --- | --- |
| `stage-0` | 0. 프로젝트 구조 | 5 | core 테스트 0 |
| `stage-1-1` | 1-1. 좌표와 타일 맵 | 4 | core 테스트 4 |
| `stage-1-2` | 1-2. 플레이어와 턴 | 4 | core 테스트 7 |
| (`exp/refs`) | 2-1. 일부러 실패하기 | 3 | 브랜치. 아래 참고 |
| `stage-2-2` | 2-2. 세대 번호 아레나 | 4 | core 테스트 7 |
| `stage-2-3` | 2-3. 컴포넌트 저장소 | 7 | core 테스트 9 |
| `stage-2-4` | 2-4. 두 엔티티 수정, 이벤트 큐 | 8 | core 테스트 20 |
| `stage-3-1` | 3-1. 턴 상태 머신 | 4 | core 테스트 22 |
| `stage-3-2` | 3-2. 몬스터 AI (enum) | 6 | core 테스트 26 |
| (`exp/dyn-ai`) | 3-2. 구현 B (`Box<dyn Brain>`) | 5 | 브랜치. core 테스트 27 |
| `stage-3-3` | 3-3. 시야와 길찾기 | 10 | core 테스트 37 |
| `stage-3-4` | 3-4. 아이템과 인벤토리 | 9 | core 테스트 45 |
| `stage-4-1` | 4-1. BSP 던전 | 6 | core 테스트 56 |
| `stage-4-2` | 4-2. 데이터 파일과 에러 타입 | 10 | core 테스트 64 |
| `stage-4-3` | 4-3. 저장과 불러오기 | 4 | core 테스트 69 |
| `stage-5-1` | 5-1. 백그라운드 층 생성 | 9 | core 테스트 78 |
| `stage-5-2` | 5-2. 헤드리스 시뮬레이션 | 6 | core 78, sim 3 |
| `stage-6` | 6. 정리 (문서 주석) | 2 | core 78, sim 3 |

마지막 `[meta]` 커밋은 이 README 와 `play.sh` 입니다 (재생에서 제외됩니다).

모든 태그에서 `rogue-core` 의 `cargo test`, `cargo clippy --all-targets -- -D warnings`, `rogue`(와 `rogue-sim`) 의 `cargo clippy -- -D warnings` 가 통과합니다.

## 재생: play.sh

```bash
./play.sh              # main 전체를 처음부터
./play.sh 2-3          # 2-3 단계만 (stage-2-2..stage-2-3)
./play.sh exp/refs     # 실험 브랜치만 (main 에서 갈라진 지점부터)
SPEED=30 ./play.sh 3-3 # 타이핑 속도 바꾸기 (기본 60)
PRINT=1 ./play.sh 2-3  # 실행하지 않고 gitlogue 명령만 출력
```

내부적으로는 이렇게 부릅니다.

```bash
gitlogue --commit stage-2-2..stage-2-3 --order asc --speed 60 --ignore play.sh --ignore README.md
```

- `0` 단계와 인자 없는 전체 재생은 루트 커밋(`cargo new` 결과물)부터의 범위라서 **루트 커밋 자체는 재생되지 않습니다.** `cargo new` 는 직접 실행하면 됩니다. 두 번째 커밋(`cargo add` 결과의 `Cargo.toml`)부터 나옵니다.
- `Cargo.lock` 은 `.gitignore` 로 빼 두었습니다 (긴 lock 파일이 타이핑되지 않게). 빌드하면 각 패키지 폴더에 생깁니다.

## gitlogue 키

| 키 | 동작 |
| --- | --- |
| `Space` | 일시정지 / 계속 |
| `h` / `l` | 한 줄 뒤로 / 앞으로 |
| `H` / `L` | 변경 단위(hunk) 뒤로 / 앞으로 |
| `p` / `n` | 이전 / 다음 커밋 |
| `q` | 종료 |

## 실험 브랜치

main 은 선형이고, 일부러 실패하는 실험은 해당 지점에서 갈라진 브랜치에 있습니다.

- `exp/refs` (stage-1-2 에서 갈라짐, 가이드 2-1): `rogue-core/src/exp.rs`
  - 시도 1: 몬스터가 `&'a Player` 를 들고 있는 설계. `cargo test` 가 `E0594` 로 **컴파일 실패**합니다.
  - 시도 2: `Rc<RefCell<Player>>`. 컴파일은 되지만 `cargo test monsters_hit` 가 `RefCell already borrowed` 로 **실행 중 패닉**합니다.
- `exp/dyn-ai` (stage-3-2 에서 갈라짐, 가이드 3-2 구현 B): AI 를 `Box<dyn Brain>` 으로 구현한 비교용 버전. 이 브랜치는 컴파일, 테스트, clippy 가 통과합니다. 4-3 에서 `Serialize` 를 붙일 수 없어 main 은 enum 을 남겼습니다.

## 실행 위치

- `rogue`: `cd rogue && cargo run` (창이 뜹니다). 4-2 부터 `../data/defs.ron` 을 읽고, 4-3 부터 `rogue/save.ron` 에 저장합니다.
- `rogue-sim`: `cd rogue-sim && cargo run --release -- 1000`, 테스트도 `rogue-sim` 폴더에서 `cargo test`.
