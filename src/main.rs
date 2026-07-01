use regex::Fsm;

fn main() {
    let fsm = Fsm::compile("a+bc");
    fsm.dump();

    let inputs = vec!["Hello, world!", "abc", "bc", "bbc", "aaaabcd"];
    for input in inputs.iter() {
        println!("{:?} => {:?}", input, fsm.match_str(input));
    }
}
