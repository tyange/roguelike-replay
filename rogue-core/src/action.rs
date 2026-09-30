use crate::geom::Dir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Wait,
    Move(Dir),
    Pickup,
    /// 계단을 내려간다
    Descend,
}
