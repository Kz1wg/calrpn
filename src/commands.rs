// コマンド定義表
// 入力トークン -> 演算 への変換 (parse) と、ヘルプ表示はすべてこの表から生成する。
// コマンドを追加するときは COMMANDS に1行追加するだけでよい。
use crate::calcrpn::{BinomialFunc, Constant, Expr, Memorize, MonomialFunc, OperateStack};

// ヘルプ表示の分類
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Category {
    Binomial,
    Monomial,
    Constant,
    Stack,
    Mode,
    Memory,
    Tvm,
}

impl Category {
    // ヘルプに表示する順序
    const ALL: [Category; 7] = [
        Category::Binomial,
        Category::Monomial,
        Category::Constant,
        Category::Stack,
        Category::Mode,
        Category::Memory,
        Category::Tvm,
    ];

    fn title(self) -> &'static str {
        match self {
            Category::Binomial => "Binomial (Y X op)",
            Category::Monomial => "Monomial (X op)",
            Category::Constant => "Constant",
            Category::Stack => "Stack",
            Category::Mode => "Angle Mode",
            Category::Memory => "Memory",
            Category::Tvm => "Time Value of Money (monthly)",
        }
    }
}

// コマンドが表す動作 (static な表に置けるよう Copy な値だけで構成する)
#[derive(Debug, Clone, Copy)]
pub enum Action {
    Binomial(BinomialFunc),
    Monomial(MonomialFunc),
    Const(Constant),
    Stack(OperateStack),
    MemoClear,
    // 以下の4つはキー待ちになり、次のトークンをキーとして受け取る
    MemoRecall,
    MemoStore,
    MemoDelete,
    Tvm,
    // X を TVM 用の固定キーに保存する (tn, tpv など)
    TvmStore(&'static str),
}

impl Action {
    pub fn to_expr(self) -> Expr {
        match self {
            Action::Binomial(f) => Expr::Binomial(f),
            Action::Monomial(f) => Expr::Monomial(f),
            Action::Const(c) => Expr::Const(c),
            Action::Stack(op) => Expr::Opstack(op),
            Action::MemoClear => Expr::Memo(Memorize::Clear),
            Action::MemoRecall => Expr::Memo(Memorize::Recall(None)),
            Action::MemoStore => Expr::Memo(Memorize::Store(None)),
            Action::MemoDelete => Expr::Memo(Memorize::Delete(None)),
            Action::Tvm => Expr::Memo(Memorize::Tvm(None)),
            Action::TvmStore(key) => Expr::Memo(Memorize::Store(Some(key.to_string()))),
        }
    }
}

pub struct Command {
    // 先頭が代表名、残りは別名。入力は小文字化して照合するので小文字で書くこと
    pub names: &'static [&'static str],
    pub action: Action,
    pub category: Category,
    pub help: &'static str,
}

const fn cmd(
    names: &'static [&'static str],
    action: Action,
    category: Category,
    help: &'static str,
) -> Command {
    Command {
        names,
        action,
        category,
        help,
    }
}

use Action as A;
use Category as C;

pub static COMMANDS: &[Command] = &[
    // 二項演算
    cmd(&["+"], A::Binomial(BinomialFunc::Add), C::Binomial, "1 2 + -> 1 + 2"),
    cmd(&["-"], A::Binomial(BinomialFunc::Subtract), C::Binomial, "1 2 - -> 1 - 2"),
    cmd(&["*"], A::Binomial(BinomialFunc::Multiply), C::Binomial, "1 2 * -> 1 * 2"),
    cmd(&["/"], A::Binomial(BinomialFunc::Divide), C::Binomial, "1 2 / -> 1 / 2"),
    cmd(&["%"], A::Binomial(BinomialFunc::Mod), C::Binomial, "15 4 % -> 15 mod 4"),
    cmd(&["^", "pow"], A::Binomial(BinomialFunc::Pow), C::Binomial, "3 2 ^ -> 3 ^ 2"),
    cmd(&["npr", "perm"], A::Binomial(BinomialFunc::NPr), C::Binomial, "10 2 npr -> 10P2"),
    cmd(&["ncr", "comb"], A::Binomial(BinomialFunc::NCr), C::Binomial, "10 2 ncr -> 10C2"),
    // 単項演算
    cmd(&["sqrt"], A::Monomial(MonomialFunc::Sqrt), C::Monomial, "9 sqrt -> sqrt(9)"),
    cmd(&["log"], A::Monomial(MonomialFunc::Log), C::Monomial, "100 log -> log10(100)"),
    cmd(&["ln"], A::Monomial(MonomialFunc::Ln), C::Monomial, "10 ln -> ln(10)"),
    cmd(&["sin"], A::Monomial(MonomialFunc::Sin), C::Monomial, "30 sin -> sin(30)"),
    cmd(&["cos"], A::Monomial(MonomialFunc::Cos), C::Monomial, "60 cos -> cos(60)"),
    cmd(&["tan"], A::Monomial(MonomialFunc::Tan), C::Monomial, "45 tan -> tan(45)"),
    cmd(&["asin"], A::Monomial(MonomialFunc::ASin), C::Monomial, "0.5 asin -> asin(0.5)"),
    cmd(&["acos"], A::Monomial(MonomialFunc::ACos), C::Monomial, "0.5 acos -> acos(0.5)"),
    cmd(&["atan"], A::Monomial(MonomialFunc::ATan), C::Monomial, "1 atan -> atan(1)"),
    cmd(&["todeg"], A::Monomial(MonomialFunc::ToDeg), C::Monomial, "pi todeg -> radians to degrees"),
    cmd(&["torad"], A::Monomial(MonomialFunc::ToRad), C::Monomial, "180 torad -> degrees to radians"),
    cmd(&["abs"], A::Monomial(MonomialFunc::Abs), C::Monomial, "3+4i abs -> |3+4i|"),
    cmd(
        &["!", "n!", "fact", "factorial"],
        A::Monomial(MonomialFunc::Factorial),
        C::Monomial,
        "5 ! -> 5!",
    ),
    cmd(
        &["topolar", "topol", "polar", "pol"],
        A::Monomial(MonomialFunc::ToPolar),
        C::Monomial,
        "3+4i topolar -> r + θi (polar form)",
    ),
    cmd(
        &["torec", "torect", "rec", "rect"],
        A::Monomial(MonomialFunc::ToRec),
        C::Monomial,
        "5+53.13i torec -> x + yi (rectangular)",
    ),
    // 定数
    cmd(&["pi"], A::Const(Constant::Pi), C::Constant, "3.14159265358979323846"),
    cmd(&["e"], A::Const(Constant::E), C::Constant, "2.71828182845904523536"),
    // スタック操作
    cmd(&["sw", "swap"], A::Stack(OperateStack::Swap), C::Stack, "swap X and Y"),
    cmd(&["cl", "clear"], A::Stack(OperateStack::Clear), C::Stack, "clear stack"),
    cmd(&["dl", "del", "delete"], A::Stack(OperateStack::Delete), C::Stack, "delete X"),
    cmd(&["rup", "rollup"], A::Stack(OperateStack::RollUp), C::Stack, "roll up stack"),
    cmd(&["rdn", "rolldown"], A::Stack(OperateStack::RollDown), C::Stack, "roll down stack"),
    cmd(&["sum"], A::Stack(OperateStack::Sum), C::Stack, "sum all stack"),
    // 角度モード
    cmd(&["deg"], A::Stack(OperateStack::Deg), C::Mode, "set degree mode"),
    cmd(&["rad"], A::Stack(OperateStack::Rad), C::Mode, "set radian mode"),
    // メモリ
    cmd(&["sto", "to"], A::MemoStore, C::Memory, "5 sto x -> store X as x"),
    cmd(&["rcl"], A::MemoRecall, C::Memory, "rcl x (or just x) -> recall x"),
    cmd(&["mdel", "mdl"], A::MemoDelete, C::Memory, "mdel x -> delete x"),
    cmd(&["mc", "mcl"], A::MemoClear, C::Memory, "clear all memory"),
    // TVM
    cmd(&["tn"], A::TvmStore("n"), C::Tvm, "store X as n (number of periods)"),
    cmd(&["tiyr"], A::TvmStore("iyr"), C::Tvm, "store X as iyr (annual rate %)"),
    cmd(&["tpv"], A::TvmStore("pv"), C::Tvm, "store X as pv (present value)"),
    cmd(&["tpmt"], A::TvmStore("pmt"), C::Tvm, "store X as pmt (payment)"),
    cmd(&["tfv"], A::TvmStore("fv"), C::Tvm, "store X as fv (future value)"),
    cmd(&["tvm", "tv"], A::Tvm, C::Tvm, "tvm pmt -> solve pmt from the other four"),
];

// main 側で処理するアプリコマンド (ヘルプ表示用)
const APP_COMMANDS: &[(&str, &str)] = &[
    ("fix <n>", "set decimal places"),
    ("clv <key>", "delete memory key"),
    ("undo", "undo last operation"),
    ("clh, clearhist", "clear input history"),
    ("help", "show this help"),
    ("quit, q", "quit"),
];

// 名前(小文字)からコマンドを探す
pub fn find(name: &str) -> Option<&'static Command> {
    COMMANDS.iter().find(|c| c.names.contains(&name))
}

pub fn help_text() -> String {
    let mut lines = Vec::new();
    for category in Category::ALL {
        lines.push(format!("[{}]", category.title()));
        for c in COMMANDS.iter().filter(|c| c.category == category) {
            lines.push(format!("  {:<26}{}", c.names.join(", "), c.help));
        }
    }
    lines.push("[App]".to_string());
    for (names, help) in APP_COMMANDS {
        lines.push(format!("  {names:<26}{help}"));
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calcrpn::CalcNum;
    use std::collections::HashSet;

    #[test]
    fn command_names_are_unique() {
        let mut seen = HashSet::new();
        for c in COMMANDS {
            for name in c.names {
                assert!(seen.insert(*name), "duplicate command name: {name}");
            }
        }
    }

    #[test]
    fn command_names_are_valid() {
        for c in COMMANDS {
            for name in c.names {
                // 入力は小文字化して照合するため
                assert_eq!(*name, name.to_lowercase(), "must be lowercase: {name}");
                // 数値として解釈される名前は到達不能になるため
                assert!(name.parse::<CalcNum>().is_err(), "parsed as number: {name}");
            }
        }
    }

    #[test]
    fn help_contains_all_commands() {
        let help = help_text();
        for c in COMMANDS {
            assert!(help.contains(c.names.join(", ").as_str()));
        }
    }
}
