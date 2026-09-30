use crate::Point;

pub struct Player {
    pub pos: Point,
    pub hp: i32,
}

pub struct Monster<'a> {
    pub pos: Point,
    pub target: &'a Player,
}

pub struct Game<'a> {
    pub player: Player,
    pub monsters: Vec<Monster<'a>>,
}

impl<'a> Game<'a> {
    /// 모든 몬스터가 자기 목표(플레이어)의 체력을 1 깎는다
    pub fn monsters_attack(&mut self) {
        for m in &self.monsters {
            m.target.hp -= 1; // 컴파일 에러: 공유 참조(&)로는 바꿀 수 없다
        }
    }
}
