use crate::action::{FsmAction, FSM_BOL, FSM_COLUMN_SIZE, FSM_ENDLINE};
use crate::column::FsmColumn;
use crate::fsm::Fsm;

pub struct Compiler {
    fsm: Fsm,
}

impl Compiler {
    pub fn compile(src: &str) -> Fsm {
        let mut c = Self {
            fsm: Fsm { cs: Vec::new() },
        };
        c.fsm.cs.push(FsmColumn::new());
        c.compile_alts(src);
        c.fsm
    }

    fn compile_alts(&mut self, src: &str) {
        let parts = split_alts(src);
        if parts.len() == 1 {
            self.compile_seq(parts[0]);
            return;
        }

        let fork = self.fsm.cs.len();
        self.fsm.cs.push(FsmColumn::new());
        let mut spans = Vec::new();

        for part in parts {
            let entry = self.fsm.cs.len();
            self.fsm.cs[fork].eps.push(entry);
            let start = self.fsm.cs.len();
            self.compile_seq(part);
            spans.push((start, self.fsm.cs.len()));
        }

        let done = self.fsm.cs.len();
        let mut col = FsmColumn::new();
        col.ts[FSM_ENDLINE] = FsmAction {
            next: done + 1,
            offset: 0,
        };
        self.fsm.cs.push(col);
        for (start, end) in spans {
            remap_branch(&mut self.fsm, start, end, done);
        }
    }

    fn compile_seq(&mut self, src: &str) {
        let mut chars = src.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '^' => self.push_bol(),
                '$' => self.push_eol(),
                '.' => self.push_dot(),
                '(' => {
                    let inner = read_until(&mut chars, ')');
                    self.compile_group(&inner, &mut chars);
                }
                '[' => {
                    let (negated, body) = read_class(&mut chars);
                    self.push_class(&body, negated);
                }
                '{' => {
                    let spec = read_until(&mut chars, '}');
                    self.apply_count(&spec);
                }
                '*' | '+' | '?' => self.apply_quant(c),
                '|' | ')' => unreachable!(),
                '\\' => {
                    let esc = chars.next().unwrap_or('\\');
                    self.push_escape(esc);
                }
                _ => self.push_char(c),
            }
        }
    }

    fn compile_group(&mut self, inner: &str, chars: &mut std::iter::Peekable<std::str::Chars>) {
        let parts = split_alts(inner);
        let fork = if parts.len() > 1 {
            Some(self.fsm.cs.len())
        } else {
            None
        };
        if parts.len() == 1 {
            self.compile_seq(parts[0]);
        } else {
            self.fsm.cs.push(FsmColumn::new());
            let mut spans = Vec::new();
            for part in parts {
                let entry = self.fsm.cs.len();
                self.fsm.cs[fork.unwrap()].eps.push(entry);
                let start = self.fsm.cs.len();
                self.compile_seq(part);
                spans.push((start, self.fsm.cs.len()));
            }
            let done = self.fsm.cs.len();
            let mut col = FsmColumn::new();
            col.ts[FSM_ENDLINE] = FsmAction {
                next: done + 1,
                offset: 0,
            };
            self.fsm.cs.push(col);
            for (start, end) in spans {
                remap_branch(&mut self.fsm, start, end, done);
            }
            if let Some(q) = chars.peek().copied() {
                if q == '*' || q == '+' || q == '?' || q == '{' {
                    let q = chars.next().unwrap();
                    if q == '{' {
                        let spec = read_until(chars, '}');
                        self.apply_group_count(fork.unwrap(), done, &spec);
                    } else {
                        self.apply_group_quant(fork.unwrap(), done, q);
                    }
                    return;
                }
            }
            return;
        }
        if let Some(q) = chars.peek().copied() {
            if q == '*' || q == '+' || q == '?' || q == '{' {
                let q = chars.next().unwrap();
                if q == '{' {
                    let spec = read_until(chars, '}');
                    self.apply_count(&spec);
                } else {
                    self.apply_quant(q);
                }
            }
        }
    }

    fn apply_group_quant(&mut self, fork: usize, done: usize, q: char) {
        match q {
            '+' => {
                self.fsm.cs[done].eps.push(fork);
            }
            '*' => {
                self.fsm.cs[done].eps.push(fork);
                self.fsm.cs[fork].eps.push(done);
            }
            '?' => {
                self.fsm.cs[fork].eps.push(done);
            }
            _ => unreachable!(),
        }
    }

    fn apply_group_count(&mut self, fork: usize, done: usize, spec: &str) {
        let (min, max) = parse_count(spec);
        if min == 0 {
            self.fsm.cs[fork].eps.push(done);
        }
        if max == 0 {
            return;
        }
        if min > 1 {
            for _ in 1..min {
                self.fsm.cs[done].eps.push(fork);
            }
        }
        if max == usize::MAX {
            self.fsm.cs[done].eps.push(fork);
            self.fsm.cs[fork].eps.push(done);
            return;
        }
        for _ in 0..max.saturating_sub(min.max(1)) {
            self.fsm.cs[done].eps.push(fork);
            self.fsm.cs[fork].eps.push(done);
        }
    }

    fn push_char(&mut self, c: char) {
        let mut col = FsmColumn::new();
        col.ts[c as usize] = FsmAction {
            next: self.fsm.cs.len() + 1,
            offset: 1,
        };
        self.fsm.cs.push(col);
    }

    fn push_escape(&mut self, esc: char) {
        match esc {
            'd' => self.fill_set(digit_set(), false),
            'w' => self.fill_set(word_set(), false),
            's' => self.fill_set(space_set(), false),
            'D' => self.fill_set(digit_set(), true),
            'W' => self.fill_set(word_set(), true),
            'S' => self.fill_set(space_set(), true),
            _ => self.push_char(esc),
        }
    }

    fn push_dot(&mut self) {
        let mut col = FsmColumn::new();
        for i in 32..127 {
            col.ts[i] = FsmAction {
                next: self.fsm.cs.len() + 1,
                offset: 1,
            };
        }
        self.fsm.cs.push(col);
    }

    fn push_bol(&mut self) {
        let mut col = FsmColumn::new();
        col.ts[FSM_BOL] = FsmAction {
            next: self.fsm.cs.len() + 1,
            offset: 0,
        };
        self.fsm.cs.push(col);
    }

    fn push_eol(&mut self) {
        let mut col = FsmColumn::new();
        col.ts[FSM_ENDLINE] = FsmAction {
            next: self.fsm.cs.len() + 1,
            offset: 0,
        };
        self.fsm.cs.push(col);
    }

    fn push_class(&mut self, body: &str, negated: bool) {
        let set = parse_class_body(body);
        self.fill_set(set, negated);
    }

    fn fill_set(&mut self, set: Vec<usize>, negated: bool) {
        let mut col = FsmColumn::new();
        let next = self.fsm.cs.len() + 1;
        if negated {
            for i in 32..127 {
                if !set.contains(&i) {
                    col.ts[i] = FsmAction { next, offset: 1 };
                }
            }
        } else {
            for i in set {
                col.ts[i] = FsmAction { next, offset: 1 };
            }
        }
        self.fsm.cs.push(col);
    }

    fn apply_quant(&mut self, q: char) {
        if self.fsm.cs.len() <= 1 {
            return;
        }
        let n = self.fsm.cs.len();
        match q {
            '*' => {
                for t in self.fsm.cs.last_mut().unwrap().ts.iter_mut() {
                    if t.next == n {
                        t.next = n - 1;
                    } else if t.next == 0 {
                        t.next = n;
                        t.offset = 0;
                    } else {
                        unreachable!();
                    }
                }
            }
            '+' => {
                self.fsm.cs.push(self.fsm.cs.last().unwrap().clone());
                for t in self.fsm.cs.last_mut().unwrap().ts.iter_mut() {
                    if t.next == n {
                    } else if t.next == 0 {
                        t.next = n + 1;
                        t.offset = 0;
                    } else {
                        unreachable!();
                    }
                }
            }
            '?' => {
                for t in self.fsm.cs.last_mut().unwrap().ts.iter_mut() {
                    if t.next == n {
                    } else if t.next == 0 {
                        t.next = n;
                        t.offset = 0;
                    } else {
                        unreachable!();
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn apply_count(&mut self, spec: &str) {
        if self.fsm.cs.len() <= 1 {
            return;
        }
        let (min, max) = parse_count(spec);
        if min == 0 {
            if max == 0 {
                return;
            }
            for i in 0..max {
                if i > 0 {
                    let n = self.fsm.cs.len();
                    self.fsm.cs.push(self.fsm.cs.last().unwrap().clone());
                    let m = self.fsm.cs.len();
                    for t in self.fsm.cs[m - 2].ts.iter_mut() {
                        if t.next == n {
                            t.next = m - 1;
                        }
                    }
                    for t in self.fsm.cs.last_mut().unwrap().ts.iter_mut() {
                        if t.next != 0 {
                            t.next = m;
                        }
                    }
                }
                self.apply_quant('?');
            }
            if max == usize::MAX {
                self.apply_quant('*');
            }
            return;
        }
        for _ in 1..min {
            let n = self.fsm.cs.len();
            self.fsm.cs.push(self.fsm.cs.last().unwrap().clone());
            let m = self.fsm.cs.len();
            for t in self.fsm.cs[m - 2].ts.iter_mut() {
                if t.next == n {
                    t.next = m - 1;
                }
            }
            for t in self.fsm.cs.last_mut().unwrap().ts.iter_mut() {
                if t.next == n {
                    t.next = m;
                }
            }
        }
        if max == usize::MAX {
            self.apply_quant('*');
            return;
        }
        for _ in 0..max - min {
            let n = self.fsm.cs.len();
            self.fsm.cs.push(self.fsm.cs.last().unwrap().clone());
            let m = self.fsm.cs.len();
            for t in self.fsm.cs[m - 2].ts.iter_mut() {
                if t.next == n {
                    t.next = m - 1;
                }
            }
            for t in self.fsm.cs.last_mut().unwrap().ts.iter_mut() {
                if t.next != 0 {
                    t.next = m;
                }
            }
            self.apply_quant('?');
        }
    }
}

fn remap_branch(fsm: &mut Fsm, start: usize, end: usize, done: usize) {
    for col in &mut fsm.cs[start..end] {
        for t in col.ts.iter_mut() {
            if t.next == end {
                t.next = done;
            }
        }
    }
}

fn split_alts(src: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth = 0;
    let mut in_class = false;
    let mut escaped = false;
    for (i, c) in src.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match c {
            '\\' => escaped = true,
            '[' if !in_class => in_class = true,
            ']' if in_class => in_class = false,
            '(' if !in_class => depth += 1,
            ')' if !in_class => depth -= 1,
            '|' if depth == 0 && !in_class => {
                parts.push(&src[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&src[start..]);
    parts
}

fn read_until(chars: &mut std::iter::Peekable<std::str::Chars>, end: char) -> String {
    let mut out = String::new();
    let mut escaped = false;
    while let Some(c) = chars.next() {
        if escaped {
            out.push(c);
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            out.push(c);
            continue;
        }
        if c == end {
            break;
        }
        out.push(c);
    }
    out
}

fn read_class(chars: &mut std::iter::Peekable<std::str::Chars>) -> (bool, String) {
    let mut body = String::new();
    let mut negated = false;
    if chars.peek() == Some(&'^') {
        negated = true;
        chars.next();
    }
    let mut escaped = false;
    while let Some(c) = chars.next() {
        if escaped {
            body.push(c);
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            body.push(c);
            continue;
        }
        if c == ']' {
            break;
        }
        body.push(c);
    }
    (negated, body)
}

fn parse_class_body(body: &str) -> Vec<usize> {
    let mut set = Vec::new();
    let chars: Vec<char> = body.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            i += 1;
            match chars[i] {
                'd' => set.extend(digit_set()),
                'w' => set.extend(word_set()),
                's' => set.extend(space_set()),
                c => push_sym(&mut set, c),
            }
            i += 1;
            continue;
        }
        if i + 2 < chars.len() && chars[i + 1] == '-' {
            let lo = chars[i] as usize;
            let hi = chars[i + 2] as usize;
            for j in lo..=hi {
                if j < FSM_COLUMN_SIZE && !set.contains(&j) {
                    set.push(j);
                }
            }
            i += 3;
            continue;
        }
        push_sym(&mut set, chars[i]);
        i += 1;
    }
    set
}

fn push_sym(set: &mut Vec<usize>, c: char) {
    let idx = c as usize;
    if idx < FSM_COLUMN_SIZE && !set.contains(&idx) {
        set.push(idx);
    }
}

fn digit_set() -> Vec<usize> {
    (b'0' as usize..=b'9' as usize).collect()
}

fn word_set() -> Vec<usize> {
    let mut set = digit_set();
    for c in 'a'..='z' {
        push_sym(&mut set, c);
    }
    for c in 'A'..='Z' {
        push_sym(&mut set, c);
    }
    push_sym(&mut set, '_');
    set
}

fn space_set() -> Vec<usize> {
    vec![32, 9, 10, 13]
}

fn parse_count(spec: &str) -> (usize, usize) {
    if spec.contains(',') {
        let parts: Vec<&str> = spec.split(',').collect();
        let min = parts[0].parse().unwrap_or(0);
        if parts.len() > 1 && !parts[1].is_empty() {
            (min, parts[1].parse().unwrap_or(min))
        } else {
            (min, usize::MAX)
        }
    } else {
        let n: usize = spec.parse().unwrap_or(0);
        (n, n)
    }
}
