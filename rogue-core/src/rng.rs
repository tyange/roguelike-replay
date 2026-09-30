use serde::{Deserialize, Serialize};

/// 시드 하나로 항상 같은 수열을 내는 난수 생성기 (splitmix64).
/// 일부러 `Copy`를 붙이지 않았다: 복사하면 두 곳이 똑같은 수열을 받는다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// `lo` 이상 `hi` 미만의 정수. `hi <= lo`이면 `lo`
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        let span = u64::try_from(hi.saturating_sub(lo)).unwrap_or(0);
        if span == 0 {
            return lo;
        }
        // span 이 i32 범위 안이라서 나머지도 항상 i32 에 들어간다
        lo + i32::try_from(self.next_u64() % span).unwrap_or(0)
    }

    /// `percent`% 확률로 true
    pub fn chance(&mut self, percent: i32) -> bool {
        self.range(0, 100) < percent
    }

    /// 슬라이스에서 하나를 고른다. 빈 슬라이스면 None. 돌려주는 참조는 슬라이스를 빌린 것이다
    pub fn pick<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        let len = i32::try_from(items.len()).unwrap_or(i32::MAX);
        let index = usize::try_from(self.range(0, len)).ok()?;
        items.get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn different_seed_different_sequence() {
        assert_ne!(Rng::new(1).next_u64(), Rng::new(2).next_u64());
    }

    #[test]
    fn range_stays_inside_bounds() {
        let mut rng = Rng::new(7);
        for _ in 0..1000 {
            let n = rng.range(-3, 4);
            assert!((-3..4).contains(&n));
        }
        assert_eq!(rng.range(5, 5), 5);
        assert_eq!(rng.range(5, 2), 5);
    }
}
