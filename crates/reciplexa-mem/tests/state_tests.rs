use reciplexa_mem::reg::Reg;
use reciplexa_mem::state::{OwnMap, OwnState};

#[test]
fn default_state_is_uninit() {
    let map = OwnMap::new();
    assert_eq!(map.get(Reg(0)), OwnState::Uninit);
}

#[test]
fn define_and_drop() {
    let mut map = OwnMap::new();
    map.define(Reg(0));
    assert_eq!(map.get(Reg(0)), OwnState::Alive);
    map.drop_reg(Reg(0));
    assert_eq!(map.get(Reg(0)), OwnState::Dropped);
}

#[test]
fn move_from_invalidates_source() {
    let mut map = OwnMap::new();
    map.define(Reg(0));
    map.move_from(Reg(0), Reg(1));
    assert_eq!(map.get(Reg(0)), OwnState::Moved);
    assert_eq!(map.get(Reg(1)), OwnState::Alive);
}

#[test]
fn alive_regs_lists_only_alive() {
    let mut map = OwnMap::new();
    map.define(Reg(0));
    map.define(Reg(1));
    map.drop_reg(Reg(1));
    let alive = map.alive_regs();
    assert_eq!(alive, vec![Reg(0)]);
}

#[test]
fn merge_join_combines_branches() {
    let mut left = OwnMap::new();
    left.define(Reg(0));
    left.drop_reg(Reg(1));
    let mut right = OwnMap::new();
    right.define(Reg(0));
    right.define(Reg(1));
    let merged = left.merge_join(&right);
    assert_eq!(merged.get(Reg(0)), OwnState::Alive);
    assert_eq!(merged.get(Reg(1)), OwnState::Alive);
}

#[test]
fn merge_join_dropped_both() {
    let mut left = OwnMap::new();
    left.define(Reg(0));
    left.drop_reg(Reg(0));
    let mut right = OwnMap::new();
    right.define(Reg(0));
    right.drop_reg(Reg(0));
    let merged = left.merge_join(&right);
    assert_eq!(merged.get(Reg(0)), OwnState::Dropped);
}
