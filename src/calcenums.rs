use core::f64;
use std::collections::BTreeMap;
use std::collections::VecDeque;

use crate::StackData;

// 演算時要素の列挙型
#[derive(Debug)]
pub enum Expr {
    Numbers(f64),
    Binomial(BinomialFunc),
    Monomial(MonomialFunc),
    Const(Constant),
    Opstack(OperateStack),
    Memo(Memorize),
}
// 二項演算の列挙型
#[derive(Debug)]
pub enum BinomialFunc {
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,
    Pow,
    NCr,
    NPr,
}

// 単項演算の列挙型
#[derive(Debug)]
pub enum MonomialFunc {
    Sqrt,
    Log,
    Ln,
    Sin,
    Cos,
    Tan,
    ASin,
    ACos,
    ATan,
    ToDeg,
    ToRad,
    Factorial,
}

// 記憶領域の列挙型
#[derive(Debug)]
pub enum Memorize {
    Recall(Option<String>),
    Clear,
    Store(Option<String>),
}

// 定数の列挙型
#[derive(Debug)]
pub enum Constant {
    Pi,
    E,
}

// スタック操作の列挙型
#[derive(Debug)]
pub enum OperateStack {
    Swap,
    Clear,
    Delete,
    RollUp,
    RollDown,
    Sum,
    Deg,
    Rad,
}

// 角度モードの列挙型
#[derive(Debug)]
pub enum DegMode {
    Rad,
    Deg,
}

const STACK_SIZE: usize = 100;
// スタックの管理関数
pub fn manage_stack(
    expression: &str,
    calstack: &mut VecDeque<StackData>,
    degmode: &mut DegMode,
    memory_map: &mut BTreeMap<String, StackData>,
    memo_mode: &mut Option<Memorize>,
) -> Result<(), String> {
    // 式を分割するクロージャ
    let separate_exp = |x: &str| match x.chars().last() {
        Some(c) => match c {
            '+' | '-' | '*' | '/' | '^' => {
                if x.len() > 1 {
                    let (head, tail) = x.split_at(x.len() - 1);
                    vec![head.to_string(), tail.to_string()]
                } else {
                    vec![x.to_string()]
                }
            }
            _ => vec![x.to_string()],
        },
        None => vec![],
    };
    // let mut memo_mode: Option<Memorize> = None;
    // 入力された式を空白で分割し、それぞれの要素をparse_exp関数で処理
    let items = expression
        .split_whitespace()
        .flat_map(separate_exp)
        .map(|arg: String| parse_exp(&arg, memo_mode))
        .collect::<Result<Vec<Expr>, String>>()?;

    // 式の要素を順番に処理
    for item in items {
        // 式の要素に応じて処理を分岐
        match item {
            // 記憶関連の処理
            Expr::Memo(mem) => match mem {
                Memorize::Recall(key) => {
                    if let Some(inkey) = key {
                        if let Some(val) = memory_map.get(&inkey) {
                            calstack.push_back(*val);
                        } else {
                            return Err("No Key".to_string());
                        }
                    }
                }
                Memorize::Clear => {
                    memory_map.clear();
                }
                Memorize::Store(key) => {
                    if let Some(inkey) = key {
                        if let Some(val) = calstack.pop_back() {
                            memory_map.insert(inkey, val);
                            calstack.push_back(val);
                        } else {
                            return Err("Stack is Empty".to_string());
                        }
                    }
                }
            },
            // 数値の場合
            Expr::Numbers(data) => calstack.push_back(StackData::Number(data)),
            // 二項演算の場合
            Expr::Binomial(b_func) => {
                let (exex, ex) = get_two_item(calstack)?;
                match (exex, ex) {
                    (StackData::Number(exex), StackData::Number(ex)) => {
                        let result = match b_func {
                            BinomialFunc::Add => exex + ex,
                            BinomialFunc::Subtract => exex - ex,
                            BinomialFunc::Multiply => exex * ex,
                            BinomialFunc::Divide => {
                                if ex == 0f64 {
                                    calstack.push_back(StackData::Number(exex));
                                    calstack.push_back(StackData::Number(ex));
                                    return Err("Zero Divided Error".to_string());
                                } else {
                                    exex / ex
                                }
                            }
                            BinomialFunc::Mod => exex % ex,
                            BinomialFunc::Pow => exex.powf(ex),
                            BinomialFunc::NPr => {
                                if exex < ex
                                    || exex < 0.0
                                    || ex < 0.0
                                    || !is_integer(exex)
                                    || !is_integer(ex)
                                {
                                    calstack.push_back(StackData::Number(exex));
                                    calstack.push_back(StackData::Number(ex));
                                    return Err("Invalid Data".to_string());
                                }
                                permutation(exex, ex)
                            }
                            BinomialFunc::NCr => {
                                if exex < ex
                                    || exex < 0.0
                                    || ex < 0.0
                                    || !is_integer(exex)
                                    || !is_integer(ex)
                                {
                                    calstack.push_back(StackData::Number(exex));
                                    calstack.push_back(StackData::Number(ex));
                                    return Err("Invalid Data".to_string());
                                }
                                combination(exex, ex)
                            }
                        };
                        calstack.push_back(StackData::Number(result));
                    }
                    _ => return Err("StackData is not Number".to_string()),
                }
            }
            // 単項演算の場合
            Expr::Monomial(m_func) => {
                let ex = get_one_item(calstack)?;
                match ex {
                    StackData::Number(ex) => {
                        let result = match m_func {
                            MonomialFunc::Sqrt => ex.sqrt(),
                            MonomialFunc::Log => ex.log(10.0),
                            MonomialFunc::Ln => ex.ln(),
                            MonomialFunc::Sin => {
                                let ex = to_rad_item(ex, degmode);
                                ex.sin()
                            }
                            MonomialFunc::Cos => {
                                let ex = to_rad_item(ex, degmode);
                                ex.cos()
                            }
                            MonomialFunc::Tan => {
                                let ex = to_rad_item(ex, degmode);
                                ex.tan()
                            }
                            MonomialFunc::ASin => to_deg_item(ex.asin(), degmode),
                            MonomialFunc::ACos => to_deg_item(ex.acos(), degmode),
                            MonomialFunc::ATan => to_deg_item(ex.atan(), degmode),
                            MonomialFunc::ToDeg => to_deg(ex),
                            MonomialFunc::ToRad => to_rad(ex),
                            MonomialFunc::Factorial => {
                                if ex < 0.0 || !is_integer(ex) {
                                    calstack.push_back(StackData::Number(ex));
                                    return Err("Invalid Data".to_string());
                                }
                                factorial(1.0, ex)
                            }
                        };
                        calstack.push_back(StackData::Number(result));
                    }
                    _ => return Err("Invalid Data".to_string()),
                }
            }
            // スタック操作・演算の場合
            Expr::Opstack(operate) => match operate {
                OperateStack::Swap => {
                    if calstack.len() < 2 {
                        return Err("Stack is too short".to_string());
                    } else {
                        let last = calstack.len() - 1;
                        calstack.swap(last, last - 1);
                    }
                }
                OperateStack::Clear => calstack.clear(),
                OperateStack::Delete => {
                    if calstack.is_empty() {
                        return Err("Stack is Empty".to_string());
                    } else {
                        calstack.pop_back();
                    }
                }
                OperateStack::RollUp => {
                    if calstack.len() < 2 {
                        return Err("Stack is too short".to_string());
                    } else {
                        let last = calstack.pop_back().unwrap();
                        calstack.push_front(last);
                    }
                }
                OperateStack::RollDown => {
                    if calstack.len() < 2 {
                        return Err("Stack is too short".to_string());
                    } else {
                        let first = calstack.pop_front().unwrap();
                        calstack.push_back(first);
                    }
                }
                OperateStack::Sum => {
                    if calstack.iter().all(StackData::is_number) {
                        let sum_result = calstack.iter().map(StackData::get_number).sum();
                        calstack.clear();
                        calstack.push_back(StackData::Number(sum_result));
                    } else {
                        return Err("Invalid Data".to_string());
                    }
                }
                OperateStack::Deg => *degmode = DegMode::Deg,
                OperateStack::Rad => *degmode = DegMode::Rad,
            },
            // 定数の場合
            Expr::Const(consts) => {
                let result = match consts {
                    Constant::Pi => f64::consts::PI,
                    Constant::E => f64::consts::E,
                };
                calstack.push_back(StackData::Number(result));
            }
        }
    }
    // スタックが一定以上になった場合、先頭の要素を削除
    if calstack.len() >= STACK_SIZE {
        calstack.pop_front();
        return Err("Stack is Full".to_string());
    };
    Ok(())
}

fn parse_exp(expression: &str, memo_mode: &mut Option<Memorize>) -> Result<Expr, String> {
    match expression.to_lowercase().parse::<f64>() {
        Ok(data) => Ok(Expr::Numbers(data)),
        Err(_) => match expression {
            "+" => Ok(Expr::Binomial(BinomialFunc::Add)),
            "-" => Ok(Expr::Binomial(BinomialFunc::Subtract)),
            "*" => Ok(Expr::Binomial(BinomialFunc::Multiply)),
            "/" => Ok(Expr::Binomial(BinomialFunc::Divide)),
            "%" => Ok(Expr::Binomial(BinomialFunc::Mod)),
            "cl" | "clear" => Ok(Expr::Opstack(OperateStack::Clear)),
            "dl" | "del" | "delete" => Ok(Expr::Opstack(OperateStack::Delete)),
            "mc" | "mcl" => Ok(Expr::Memo(Memorize::Clear)),
            "rup" | "rollup" => Ok(Expr::Opstack(OperateStack::RollUp)),
            "rdn" | "rolldown" => Ok(Expr::Opstack(OperateStack::RollDown)),
            "sw" | "swap" => Ok(Expr::Opstack(OperateStack::Swap)),
            "sin" => Ok(Expr::Monomial(MonomialFunc::Sin)),
            "cos" => Ok(Expr::Monomial(MonomialFunc::Cos)),
            "tan" => Ok(Expr::Monomial(MonomialFunc::Tan)),
            "asin" => Ok(Expr::Monomial(MonomialFunc::ASin)),
            "acos" => Ok(Expr::Monomial(MonomialFunc::ACos)),
            "atan" => Ok(Expr::Monomial(MonomialFunc::ATan)),
            "^" | "pow" => Ok(Expr::Binomial(BinomialFunc::Pow)),
            "sqrt" => Ok(Expr::Monomial(MonomialFunc::Sqrt)),
            "log" => Ok(Expr::Monomial(MonomialFunc::Log)),
            "ln" => Ok(Expr::Monomial(MonomialFunc::Ln)),
            "npr" | "perm" => Ok(Expr::Binomial(BinomialFunc::NPr)),
            "ncr" | "comb" => Ok(Expr::Binomial(BinomialFunc::NCr)),
            "n!" | "fact" | "factorial" => Ok(Expr::Monomial(MonomialFunc::Factorial)),
            "pi" => Ok(Expr::Const(Constant::Pi)),
            "e" => Ok(Expr::Const(Constant::E)),
            "sum" => Ok(Expr::Opstack(OperateStack::Sum)),
            "torad" => Ok(Expr::Monomial(MonomialFunc::ToRad)),
            "todeg" => Ok(Expr::Monomial(MonomialFunc::ToDeg)),
            "rad" => Ok(Expr::Opstack(OperateStack::Rad)),
            "deg" => Ok(Expr::Opstack(OperateStack::Deg)),
            _ => match memo_mode {
                Some(Memorize::Recall(None)) => {
                    *memo_mode = None;
                    Ok(Expr::Memo(Memorize::Recall(Some(expression.to_string()))))
                }
                Some(Memorize::Store(None)) => {
                    *memo_mode = None;
                    Ok(Expr::Memo(Memorize::Store(Some(expression.to_string()))))
                }
                _ => match expression.to_lowercase().as_str() {
                    "rcl" => {
                        *memo_mode = Some(Memorize::Recall(None));
                        Ok(Expr::Memo(Memorize::Recall(None)))
                    }
                    "sto" => {
                        *memo_mode = Some(Memorize::Store(None));
                        Ok(Expr::Memo(Memorize::Store(None)))
                    }
                    "mc" | "mcl" => {
                        *memo_mode = None;
                        Ok(Expr::Memo(Memorize::Clear))
                    }
                    _ => Err("Invalid Data".to_string()),
                },
            },
        },
    }
}

// スタックから2つの要素を取り出す
fn get_two_item(calstack: &mut VecDeque<StackData>) -> Result<(StackData, StackData), String> {
    if calstack.len() < 2 {
        Err("Stack is too short".to_string())
    } else {
        match (calstack.pop_back(), calstack.pop_back()) {
            (Some(ex), Some(exex)) => Ok((exex, ex)),
            _ => Err("Stack two pop Error".to_string()),
        }
    }
}
// スタックから1つの要素を取り出す
fn get_one_item(calstack: &mut VecDeque<StackData>) -> Result<StackData, String> {
    if calstack.is_empty() {
        Err("Stack is Empty".to_string())
    } else {
        match calstack.pop_back() {
            Some(ex) => Ok(ex),
            None => Err("Stack is Empty".to_string()),
        }
    }
}
// 角度をラジアンに変換
fn to_rad(val: f64) -> f64 {
    val / 180.0 * f64::consts::PI
}
// ラジアンを角度に変換
fn to_deg(val: f64) -> f64 {
    val / f64::consts::PI * 180.0
}
//　値をラジアンに変換する
fn to_rad_item(val: f64, mode: &DegMode) -> f64 {
    match mode {
        DegMode::Rad => val,
        DegMode::Deg => to_rad(val),
    }
}
// 値をDegreeに変換する
fn to_deg_item(val: f64, mode: &DegMode) -> f64 {
    match mode {
        DegMode::Rad => val,
        DegMode::Deg => to_deg(val),
    }
}
// 整数かどうかを判定
fn is_integer(val: f64) -> bool {
    val.fract() == 0.0
}
// 階乗計算
fn factorial(start: f64, end: f64) -> f64 {
    (start as u64..=end as u64).fold(1.0, |acc, x| acc * x as f64)
}
// 順列計算
fn permutation(n: f64, r: f64) -> f64 {
    factorial(n - r + 1.0, n)
}
// 組み合わせ計算
fn combination(n: f64, r: f64) -> f64 {
    permutation(n, r) / factorial(1.0, r)
}

#[cfg(test)]
mod tests {
    use crate::{manage_stack, DegMode};
    use core::f64;
    use std::collections::{BTreeMap, VecDeque};

    #[test]
    fn test() -> Result<(), String> {
        let test_manage = |exp| {
            let mut teststack = VecDeque::new();
            let mut test_memory = BTreeMap::new();
            let mut memomode = None;
            match manage_stack(
                exp,
                &mut teststack,
                &mut DegMode::Rad,
                &mut test_memory,
                &mut memomode,
            ) {
                Ok(_) => (),
                Err(e) => eprintln!("{e}"),
            };
            teststack[0].get_number()
        };
        assert_eq!(test_manage("2 3 +"), 5.0);
        assert_eq!(test_manage("2 3 -"), -1.0);
        assert_eq!(test_manage("2 3 *"), 6.0);
        assert_eq!(test_manage("3 2 /"), 1.5);
        assert_eq!(test_manage("3 2 ^"), 9.0);
        assert_eq!(test_manage("3 2/"), 1.5);
        assert_eq!(test_manage("3 2/ 2 * 2^"), 9.0);
        assert_eq!(test_manage("9.0 sqrt"), 3.0);
        assert_eq!(test_manage("10.0 ln"), f64::consts::LN_10);
        assert_eq!(test_manage("100.0 log"), 2.0);
        assert_eq!(test_manage("9.0 3 3 5 sum"), 20.0);
        assert!((test_manage("pi 6 / sin") - 0.5) < 1e-10);
        assert!((test_manage("pi 3 / cos") - 0.5) < 1e-10);
        assert!((test_manage("pi 4 / tan") - 1.0) < 1e-10);
        assert!((test_manage("0.5 asin") - f64::consts::PI / 6.0) < 1e-10);
        assert!((test_manage("0.5 acos") - f64::consts::PI / 3.0) < 1e-10);
        assert!((test_manage("1.0 atan") - f64::consts::PI / 4.0) < 1e-10);
        assert_eq!(test_manage("pi"), f64::consts::PI);
        assert_eq!(test_manage("e"), f64::consts::E);
        assert_eq!(test_manage("60 torad"), f64::consts::PI / 3.0);
        assert_eq!(test_manage("pi 3 / todeg"), 60.0);
        assert_eq!(test_manage("2 3 + 12 *"), 60.0);
        assert_eq!(test_manage("3 3 3 sum sqrt"), 3.0);
        assert_eq!(test_manage("10 3 npr"), 720.0);
        assert_eq!(test_manage("10 3 ncr"), 120.0);
        assert_eq!(test_manage("5 n!"), 120.0);
        Ok(())
    }
}
