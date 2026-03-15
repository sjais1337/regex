use std::ops::Range;
type FsmIndex = usize;

const FSM_COLUMN_SIZE: usize = 130;
const FSM_ENDLINE: usize = 129;


#[derive(Default, Clone, Copy)]
struct FsmAction {
    next: FsmIndex,
    offset: i32,
}

#[derive(Clone)]
struct FsmColumn {
    ts: [FsmAction; FSM_COLUMN_SIZE],
}

struct Fsm {
    cs: Vec<FsmColumn>
}

impl FsmColumn {
    fn new() -> Self {
        Self {
            ts: [Default::default(); FSM_COLUMN_SIZE]
        }
    }
}


impl Fsm {
    fn compile(src: &str) -> Self {
        let mut fsm = Self { cs: Vec::new() };
        fsm.cs.push(FsmColumn::new());

        for c in src.chars() {
            let mut col = FsmColumn::new();

            match c {
                '$' => {
                    col.ts[FSM_ENDLINE] = FsmAction{
                        next: fsm.cs.len() + 1,
                        offset: 1
                    };
                    fsm.cs.push(col);
                },
                '.' => {
                    for i in 32..127 {
                        col.ts[i] = FsmAction {
                            next: fsm.cs.len() + 1, 
                            offset: 1
                        };
                    }
                    fsm.cs.push(col);
                },
                '*' => {
                    let n = fsm.cs.len();
                    for t in fsm.cs.last_mut().unwrap().ts.iter_mut() {
                        if t.next == n {
                            t.next = n-1;
                        } else if t.next == 0 {
                            t.next = n; 
                            t.offset = 0;
                        } else {
                            unreachable!();
                        }
                    }
                },
                '+' => {
                    let n = fsm.cs.len();
                    fsm.cs.push(fsm.cs.last().unwrap().clone());
                    for t in fsm.cs.last_mut().unwrap().ts.iter_mut() {
                        if t.next == n {
                        } else if t.next == 0 {
                            t.next = n+1; 
                            t.offset = 0;
                        } else {
                            unreachable!();
                        }
                    }
                },
                _ => {
                    col.ts[c as usize] = FsmAction {
                        next: fsm.cs.len() + 1,
                        offset: 1
                    };
                    fsm.cs.push(col);
                }
            }

        }

        fsm
    }

    fn dump(&self) {
        for symbol in 0..FSM_COLUMN_SIZE {
            print!("{:03} => ", symbol);
            for col in self.cs.iter() {
                print!("({}, {})", 
                    col.ts[symbol].next,
                    col.ts[symbol].offset);
            }
            println!("");
        }
    }
    
    fn match_str(&self, input: &str) -> bool {
        let mut state = 1;
        let mut head = 0;
        let chars = input.chars().collect::<Vec<_>>();
        let n = chars.len();

        while 0 < state && state < self.cs.len() && head < n{
            let action = self.cs[state].ts[chars[head] as usize];

            state = action.next;
            head = (head as i32 + action.offset) as usize;
        }

        if state == 0 {
            return false;
        }

        if state < self.cs.len() {
            let action = self.cs[state].ts[FSM_ENDLINE];
            state = action.next;
        }

        return state >= self.cs.len();
    }
}


fn main(){
    let fsm = Fsm::compile("a+bc");
    fsm.dump();

    let inputs = vec!["Hello, world!", "abc", "bc", "bbc", "aaaabcd"];
    for input in inputs.iter() {
        println!("{:?} => {:?}", input, fsm.match_str(input));
    }
}