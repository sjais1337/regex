use crate::action::{FSM_BOL, FSM_COLUMN_SIZE, FSM_ENDLINE};
use crate::column::FsmColumn;

pub struct Fsm {
    pub cs: Vec<FsmColumn>,
}

impl Fsm {
    pub fn compile(src: &str) -> Self {
        crate::compile::Compiler::compile(src)
    }

    pub fn dump(&self) {
        for symbol in 0..FSM_COLUMN_SIZE {
            print!("{:03} => ", symbol);
            for col in self.cs.iter() {
                print!("({}, {})", col.ts[symbol].next, col.ts[symbol].offset);
            }
            println!("");
        }
    }

    fn sym_idx(c: char) -> Option<usize> {
        let idx = c as usize;
        if idx < FSM_COLUMN_SIZE {
            Some(idx)
        } else {
            None
        }
    }

    fn run_from(&self, mut state: usize, mut head: usize, chars: &[char], n: usize) -> Option<(usize, usize)> {
        while 0 < state && state < self.cs.len() && head < n {
            let idx = match Self::sym_idx(chars[head]) {
                Some(i) => i,
                None => return None,
            };
            let action = self.cs[state].ts[idx];
            if action.next == 0 {
                if self.cs[state].eps.is_empty() {
                    state = 0;
                }
                break;
            }
            state = action.next;
            head = (head as i32 + action.offset) as usize;
        }
        Some((state, head))
    }

    fn end_state(&self, state: usize) -> usize {
        if state == 0 || state >= self.cs.len() {
            return state;
        }
        self.cs[state].ts[FSM_ENDLINE].next
    }

    pub fn match_str(&self, input: &str) -> bool {
        let chars: Vec<char> = input.chars().collect();
        let n = chars.len();
        let mut stack = vec![(1usize, 0usize)];
        let mut seen = std::collections::HashSet::new();

        while let Some((start, head)) = stack.pop() {
            if start == 0 {
                continue;
            }
            if start >= self.cs.len() {
                return true;
            }
            if !seen.insert((start, head)) {
                continue;
            }

            if head == 0 {
                let bol = self.cs[start].ts[FSM_BOL];
                if bol.next != 0 {
                    stack.push((bol.next, head));
                }
            }

            for &next in self.cs[start].eps.iter() {
                stack.push((next, head));
            }

            let (state, head) = match self.run_from(start, head, &chars, n) {
                Some(v) => v,
                None => continue,
            };

            if state == 0 {
                continue;
            }
            if state >= self.cs.len() {
                return true;
            }

            for &next in self.cs[state].eps.iter() {
                stack.push((next, head));
            }

            if head >= n {
                let after = self.end_state(state);
                if after >= self.cs.len() {
                    return true;
                }
                if after != 0 && after != state {
                    stack.push((after, head));
                }
                continue;
            }

            let after = self.end_state(state);
            if after >= self.cs.len() {
                return true;
            }
            if after != 0 && after != state {
                stack.push((after, head));
            }
        }

        false
    }
}
