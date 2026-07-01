pub type FsmIndex = usize;

pub const FSM_COLUMN_SIZE: usize = 130;
pub const FSM_BOL: usize = 128;
pub const FSM_ENDLINE: usize = 129;

#[derive(Default, Clone, Copy)]
pub struct FsmAction {
    pub next: FsmIndex,
    pub offset: i32,
}
