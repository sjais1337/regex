use regex::Fsm;

fn check(pattern: &str, input: &str, expect: bool) {
    let fsm = Fsm::compile(pattern);
    let got = fsm.match_str(input);
    assert_eq!(got, expect, "pattern={:?} input={:?}", pattern, input);
}

#[test]
fn basic_literals() {
    check("abc", "abc", true);
    check("abc", "ab", false);
    check("abc", "abcd", true);
    check("", "", true);
}

#[test]
fn plus_quantifier() {
    check("a+bc", "abc", true);
    check("a+bc", "bc", false);
    check("a+bc", "aaaabc", true);
    check("a+bc", "bbc", false);
}

#[test]
fn star_quantifier() {
    check("a*b", "b", true);
    check("a*b", "ab", true);
    check("a*b", "aaaab", true);
    check("a*b", "c", false);
    check("a*", "", true);
    check("a*", "aaa", true);
}

#[test]
fn question_quantifier() {
    check("a?b", "b", true);
    check("a?b", "ab", true);
    check("a?b", "aab", false);
}

#[test]
fn dot_wildcard() {
    check("a.c", "abc", true);
    check("a.c", "a c", true);
    check("a.c", "ac", false);
}

#[test]
fn end_anchor() {
    check("ab$", "ab", true);
    check("ab$", "abc", false);
    check("a+$", "aaa", true);
    check("a+$", "aaab", false);
}

#[test]
fn start_anchor() {
    check("^ab", "ab", true);
    check("^ab", "xab", false);
    check("^a", "a", true);
}

#[test]
fn char_class() {
    check("[abc]", "a", true);
    check("[abc]", "d", false);
    check("[a-z]", "m", true);
    check("[a-z]", "A", false);
    check("[^0-9]", "a", true);
    check("[^0-9]", "5", false);
}

#[test]
fn escapes() {
    check(r"\d+", "123", true);
    check(r"\d+", "abc", false);
    check(r"\w+", "a_z9", true);
    check(r"\s+", " \t", true);
    check(r"a\.b", "a.b", true);
    check(r"a\.b", "axb", false);
}

#[test]
fn grouping() {
    check("(ab)+", "abab", true);
    check("(ab)+", "ab", true);
    check("(ab)+", "a", false);
    check("(a|b)+", "aba", true);
    check("(a|b)+", "ab", true);
}

#[test]
fn counted_repetition() {
    check("a{3}", "aaa", true);
    check("a{3}", "aa", false);
    check("a{2,4}", "aaa", true);
    check("a{2,4}", "a", false);
    check("a{1,}", "aaaa", true);
    check("a{0,2}", "", true);
    check("a{0,2}", "aa", true);
    check("a{0,2}", "aaa", true);
}

#[test]
fn alternation() {
    check("a|b", "a", true);
    check("a|b", "b", true);
    check("a|b", "c", false);
    check("cat|dog", "cat", true);
    check("cat|dog", "dog", true);
    check("(ab)|(ac)", "ab", true);
    check("(ab)|(ac)", "ac", true);
}

#[test]
fn edge_cases() {
    check(".*", "", true);
    check(".*", "anything!", true);
    check("^$", "", true);
    check("^$", "a", false);
    check(r"\*", "*", true);
    check("a?", "", true);
    check("[\\]]", "]", true);
}

#[test]
fn non_ascii_input() {
    check("a", "a", true);
    check("a", "á", false);
    check(".", "á", false);
}
