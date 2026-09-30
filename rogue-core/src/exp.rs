use std::cell::RefCell;
use std::rc::Rc;

use crate::Point;

pub struct Player {
    pub pos: Point,
    pub hp: i32,
}

pub struct Monster {
    pub pos: Point,
    pub target: Rc<RefCell<Player>>,
}

#[test]
fn monsters_hit_player() {
    let player = Rc::new(RefCell::new(Player { pos: Point::new(5, 5), hp: 10 }));
    let monsters = vec![
        Monster { pos: Point::new(6, 5), target: Rc::clone(&player) },
        Monster { pos: Point::new(4, 5), target: Rc::clone(&player) },
    ];

    // 다른 곳(예: 플레이어 위치를 읽는 로직)이 플레이어를 빌려 둔 상황
    let watching = player.borrow();

    for m in &monsters {
        m.target.borrow_mut().hp -= 1; // 실행 중 패닉: already borrowed
    }

    assert_eq!(watching.pos, Point::new(5, 5)); // 빌림이 루프 뒤까지 살아 있게 한 번 쓴다
}
