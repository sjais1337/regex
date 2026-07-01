use crate::action::{FsmAction, FSM_COLUMN_SIZE};

#[derive(Clone)]
pub struct FsmColumn {
    pub ts: [FsmAction; FSM_COLUMN_SIZE],
    pub eps: Vec<usize>,
}

impl FsmColumn {
    pub fn new() -> Self {
        Self {
            ts: [Default::default(); FSM_COLUMN_SIZE],
            eps: Vec::new(),
        }
    }
}
