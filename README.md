# roguelike-replay

「로그라이크 직접 만들기 — 서술형 가이드 (Macroquad)」를 단계별 완성 코드로 쌓아 둔 저장소입니다.
커밋 하나가 파일 하나(또는 의미 단위 하나)이고, 각 단계의 마지막 커밋에 태그가 붙어 있습니다.
gitlogue 로 커밋 diff 를 타이핑 애니메이션으로 재생하면서 따라 치는 용도입니다.

- 구조: 루트에 `.gitignore`, 옆에 나란히 놓인 독립 패키지 `rogue-core/`, `rogue/`, (5-2 부터) `rogue-sim/`, (4-2 부터) 데이터 `data/defs.ron`
- 기준: Rust 1.97+, edition 2024, macroquad 0.4.16, serde 1, ron 0.12, rayon 1.12
- 모든 단계의 코드는 완성본입니다 (`TODO` / `todo!()` 없음). 일부러 실패하는 코드는 `exp/*` 브랜치에만 있습니다.

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
