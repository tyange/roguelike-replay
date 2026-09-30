use std::collections::HashSet;
use std::path::Path;

use serde::Deserialize;

use crate::comps::{Ai, Effect, Rgb};
use crate::error::DataError;

#[derive(Debug, Clone, Deserialize)]
pub struct MonsterDef {
    pub name: String,
    pub glyph: char,
    pub color: Rgb,
    pub hp: i32,
    pub power: i32,
    pub defense: i32,
    pub vision: i32,
    pub ai: Ai,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ItemDef {
    pub name: String,
    pub glyph: char,
    pub color: Rgb,
    pub effect: Effect,
    /// 방 하나에 놓일 확률(%)
    pub chance: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Defs {
    pub monsters: Vec<MonsterDef>,
    pub items: Vec<ItemDef>,
}

fn invalid(what: impl Into<String>, reason: impl Into<String>) -> DataError {
    DataError::Invalid {
        what: what.into(),
        reason: reason.into(),
    }
}

impl Defs {
    pub fn load(path: impl AsRef<Path>) -> Result<Defs, DataError> {
        let text = std::fs::read_to_string(path)?; // io::Error → DataError::Io
        Defs::from_ron(&text)
    }

    pub fn from_ron(text: &str) -> Result<Defs, DataError> {
        let defs: Defs = ron::from_str(text)?; // SpannedError → DataError::Parse
        defs.validate()?;
        Ok(defs)
    }

    fn validate(&self) -> Result<(), DataError> {
        if self.monsters.is_empty() {
            return Err(invalid("monsters", "at least one monster is required"));
        }
        let mut seen = HashSet::new();
        for m in &self.monsters {
            let what = format!("monster \"{}\"", m.name);
            if !seen.insert(m.name.as_str()) {
                return Err(invalid(what, "duplicate name"));
            }
            if m.hp <= 0 {
                return Err(invalid(what, format!("hp must be positive (got {})", m.hp)));
            }
            if m.power < 0 || m.defense < 0 {
                return Err(invalid(what, "power and defense cannot be negative"));
            }
            if m.vision <= 0 {
                return Err(invalid(what, "vision must be positive"));
            }
        }
        let mut seen = HashSet::new();
        for i in &self.items {
            let what = format!("item \"{}\"", i.name);
            if !seen.insert(i.name.as_str()) {
                return Err(invalid(what, "duplicate name"));
            }
            if !(0..=100).contains(&i.chance) {
                return Err(invalid(what, "chance must be between 0 and 100"));
            }
            match i.effect {
                Effect::Heal(n) if n <= 0 => {
                    return Err(invalid(what, "heal amount must be positive"));
                }
                Effect::AreaDamage { amount, radius } if amount <= 0 || radius < 0 => {
                    return Err(invalid(what, "bad area damage numbers"));
                }
                Effect::Heal(_) | Effect::AreaDamage { .. } => {}
            }
        }
        Ok(())
    }
}

/// 이 프로젝트의 파일을 테스트가 그대로 읽는다
#[cfg(test)]
pub fn builtin() -> Defs {
    Defs::from_ron(include_str!("../../data/defs.ron")).expect("data/defs.ron is checked by tests")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn shipped_file_is_valid() {
        let defs = builtin();
        assert_eq!(defs.monsters.len(), 2);
        assert_eq!(defs.items.len(), 2);
    }

    const MIN: &str = r#"(monsters: [(name: "A", glyph: 'a', color: (r:1,g:2,b:3), hp: 5, power: 1, defense: 0, vision: 5, ai: Chaser)], items: [])"#;

    #[test]
    fn minimal_file_loads() {
        assert!(Defs::from_ron(MIN).is_ok());
    }

    #[test]
    fn missing_file_is_io_error_with_source() {
        let err = Defs::load("/definitely/not/here.ron").unwrap_err();
        assert!(matches!(err, DataError::Io(_)));
        assert!(err.source().is_some());
    }

    #[test]
    fn typo_is_parse_error() {
        let err = Defs::from_ron(&MIN.replace("hp:", "hpp:")).unwrap_err();
        assert!(matches!(err, DataError::Parse(_)));
        assert!(err.source().is_some());
    }

    #[test]
    fn negative_hp_is_invalid() {
        let err = Defs::from_ron(&MIN.replace("hp: 5", "hp: -5")).unwrap_err();
        match err {
            DataError::Invalid { what, reason } => {
                assert!(what.contains("\"A\""));
                assert!(reason.contains("-5"));
            }
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn display_is_readable() {
        let err = Defs::from_ron(&MIN.replace("hp: 5", "hp: 0")).unwrap_err();
        assert_eq!(
            err.to_string(),
            "monster \"A\": hp must be positive (got 0)"
        );
    }

    #[test]
    fn unknown_effect_name_is_parse_error() {
        let text = MIN.replace(
            "items: []",
            r#"items: [(name: "B", glyph: 'b', color: (r:1,g:2,b:3), effect: Explode(3), chance: 10)]"#,
        );
        let err = Defs::from_ron(&text).unwrap_err();
        assert!(matches!(err, DataError::Parse(_)));
    }

    #[test]
    fn duplicate_names_are_invalid() {
        let monster = r#"(name: "A", glyph: 'a', color: (r:1,g:2,b:3), hp: 5, power: 1, defense: 0, vision: 5, ai: Chaser)"#;
        let text = format!("(monsters: [{monster}, {monster}], items: [])");
        let err = Defs::from_ron(&text).unwrap_err();
        assert!(matches!(err, DataError::Invalid { .. }));
    }
}
