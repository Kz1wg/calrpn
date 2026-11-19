// 入力欄のシンタックスハイライト用トークン分類
// 判定は calcrpn::parse_exp と同じ優先順位 (数値 > コマンド > メモのキー) に合わせている
use crate::calcrpn::{CalcNum, Memorize};
use crate::commands::{self, Action, Category};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    Space,
    Number,
    Operator, // 二項演算
    Function, // 単項演算
    Stack,    // スタック操作・角度モード
    Memory,   // メモリ・TVM コマンド
    Constant,
    Key,     // 有効なメモのキー
    Unknown, // 評価するとエラーになる語
}

// キー待ちの種類
#[derive(Clone, Copy)]
enum Wait {
    None,
    Store,
    Recall,
    Delete,
    Tvm,
}

const TVM_KEYS: [&str; 5] = ["n", "iyr", "pv", "pmt", "fv"];

// 入力文字列を (部分文字列, 種類) の列に分ける。部分文字列を連結すると元の入力に戻る
pub fn classify<'a>(
    input: &'a str,
    memo_map: &BTreeMap<String, CalcNum>,
    memo_mode: &Option<Memorize>,
) -> Vec<(&'a str, TokenKind)> {
    let mut wait = match memo_mode {
        Some(Memorize::Store(None)) => Wait::Store,
        Some(Memorize::Recall(None)) => Wait::Recall,
        Some(Memorize::Delete(None)) => Wait::Delete,
        Some(Memorize::Tvm(None)) => Wait::Tvm,
        _ => Wait::None,
    };

    let mut result = Vec::new();
    for (chunk, is_space) in split_runs(input) {
        if is_space {
            result.push((chunk, TokenKind::Space));
            continue;
        }
        for token in split_trailing_operator(chunk) {
            let kind = classify_token(token, memo_map, &mut wait);
            result.push((token, kind));
        }
    }
    result
}

fn classify_token(token: &str, memo_map: &BTreeMap<String, CalcNum>, wait: &mut Wait) -> TokenKind {
    let lower = token.to_lowercase();
    if lower.parse::<CalcNum>().is_ok() {
        return TokenKind::Number;
    }
    if let Some(command) = commands::find(&lower) {
        match command.action {
            Action::MemoStore => *wait = Wait::Store,
            Action::MemoRecall => *wait = Wait::Recall,
            Action::MemoDelete => *wait = Wait::Delete,
            Action::Tvm => *wait = Wait::Tvm,
            _ => {}
        }
        return match command.category {
            Category::Binomial => TokenKind::Operator,
            Category::Monomial => TokenKind::Function,
            Category::Constant => TokenKind::Constant,
            Category::Stack | Category::Mode => TokenKind::Stack,
            Category::Memory | Category::Tvm => TokenKind::Memory,
        };
    }
    // コマンドでなければメモのキー
    let valid = match *wait {
        Wait::Store => true,
        Wait::Tvm => TVM_KEYS.contains(&token),
        Wait::Recall | Wait::Delete | Wait::None => memo_map.contains_key(token),
    };
    *wait = Wait::None;
    if valid {
        TokenKind::Key
    } else {
        TokenKind::Unknown
    }
}

// 空白の連続と非空白の連続に分ける (true = 空白)
fn split_runs(input: &str) -> Vec<(&str, bool)> {
    let mut runs = Vec::new();
    let mut start = 0;
    let mut current: Option<bool> = None;
    for (i, c) in input.char_indices() {
        let is_space = c.is_whitespace();
        if current.is_some_and(|s| s != is_space) {
            runs.push((&input[start..i], current.unwrap()));
            start = i;
        }
        current = Some(is_space);
    }
    if let Some(is_space) = current {
        runs.push((&input[start..], is_space));
    }
    runs
}

// "2/" のように末尾に付いた演算子を分ける (calcrpn の split_trailing_operator と同じ規則)
fn split_trailing_operator(token: &str) -> Vec<&str> {
    match token.chars().last() {
        Some('+' | '-' | '*' | '/' | '^') if token.len() > 1 => {
            let (head, tail) = token.split_at(token.len() - 1);
            vec![head, tail]
        }
        _ => vec![token],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use TokenKind::*;

    fn kinds<'a>(input: &'a str, memo_map: &BTreeMap<String, CalcNum>) -> Vec<(&'a str, TokenKind)> {
        classify(input, memo_map, &None)
            .into_iter()
            .filter(|(_, k)| *k != Space)
            .collect()
    }

    #[test]
    fn classify_basic_tokens() {
        let memo = BTreeMap::new();
        assert_eq!(
            kinds("1 2/ sqrt pi sw foo", &memo),
            [
                ("1", Number),
                ("2", Number),
                ("/", Operator),
                ("sqrt", Function),
                ("pi", Constant),
                ("sw", Stack),
                ("foo", Unknown),
            ]
        );
    }

    #[test]
    fn classify_memory_keys() {
        let mut memo = BTreeMap::new();
        memo.insert("x".to_string(), CalcNum::Number(1.0));
        assert_eq!(
            kinds("5 sto y x rcl z tvm pmt tvm abc", &memo),
            [
                ("5", Number),
                ("sto", Memory),
                ("y", Key), // sto の後は新しいキーでも有効
                ("x", Key), // 既存のキー
                ("rcl", Memory),
                ("z", Unknown), // 存在しないキー
                ("tvm", Memory),
                ("pmt", Key),
                ("tvm", Memory),
                ("abc", Unknown), // TVM の対象外
            ]
        );
    }

    #[test]
    fn waiting_from_previous_line() {
        let memo = BTreeMap::new();
        let waiting = Some(Memorize::Store(None));
        let result = classify("newkey", &memo, &waiting);
        assert_eq!(result, [("newkey", Key)]);
    }

    #[test]
    fn concatenation_restores_input() {
        let memo = BTreeMap::new();
        let input = "  1  2+ 漢字  ";
        let joined: String = classify(input, &memo, &None).iter().map(|(s, _)| *s).collect();
        assert_eq!(joined, input);
    }
}
