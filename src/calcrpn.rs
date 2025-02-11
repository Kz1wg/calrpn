use crate::StackData;
use core::f64;
use std::collections::BTreeMap;
use std::collections::VecDeque;

// 演算時要素の列挙型
#[derive(Debug)]
pub enum Expr {
    Numbers(StackData),
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
    Abs,
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

const STACK_SIZE: usize = 15;

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
                            calstack.push_back(val.clone());
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
                            memory_map.insert(inkey, val.clone());
                            calstack.push_back(val);
                        } else {
                            return Err("Stack is Empty".to_string());
                        }
                    }
                }
            },
            // 数値の場合
            Expr::Numbers(data) => calstack.push_back(data),
            // 二項演算の場合
            Expr::Binomial(b_func) => {
                let (exex, ex) = get_two_item(calstack)?;
                let result = match b_func {
                    BinomialFunc::Add => exex + ex,
                    BinomialFunc::Subtract => exex - ex,
                    BinomialFunc::Multiply => exex * ex,
                    BinomialFunc::Divide => exex / ex,
                    BinomialFunc::Mod => exex % ex,
                    BinomialFunc::Pow => match exex.pow(&ex) {
                        Ok(result) => result,
                        Err(e) => {
                            calstack.push_back(exex);
                            calstack.push_back(ex);
                            return Err(e);
                        }
                    },
                    BinomialFunc::NPr => match exex.permutation(&ex) {
                        Ok(result) => result,
                        Err(e) => {
                            calstack.push_back(exex);
                            calstack.push_back(ex);
                            return Err(e);
                        }
                    },
                    BinomialFunc::NCr => match exex.combination(&ex) {
                        Ok(result) => result,
                        Err(e) => {
                            calstack.push_back(exex);
                            calstack.push_back(ex);
                            return Err(e);
                        }
                    },
                };
                calstack.push_back(result);
            }

            // 単項演算の場合
            Expr::Monomial(m_func) => {
                let ex = get_one_item(calstack)?;
                let result = match m_func {
                    MonomialFunc::Sqrt => ex.sqrt(),
                    MonomialFunc::Log => ex.log10(),
                    MonomialFunc::Ln => ex.ln(),
                    MonomialFunc::Sin => ex.sin(degmode),
                    MonomialFunc::Cos => ex.cos(degmode),
                    MonomialFunc::Tan => ex.tan(degmode),
                    MonomialFunc::ASin => ex.asin(degmode),
                    MonomialFunc::ACos => ex.acos(degmode),
                    MonomialFunc::ATan => ex.atan(degmode),
                    MonomialFunc::ToDeg => ex.to_deg()?,
                    MonomialFunc::ToRad => ex.to_rad()?,
                    MonomialFunc::Factorial => match ex.factorial() {
                        Ok(result) => result,
                        Err(e) => {
                            calstack.push_back(ex);
                            return Err(e);
                        }
                    },
                    MonomialFunc::Abs => ex.abs(),
                };
                calstack.push_back(result);
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
                    if calstack.iter().all(StackData::is_realnumber) {
                        let sum_result = calstack.iter().map(|x| x.get_realnumber().unwrap()).sum();
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
    // スタックが一定以上になった場合、先頭の要素 wを削除
    if calstack.len() >= STACK_SIZE {
        calstack.pop_front();
        return Err("Stack is Full".to_string());
    };
    Ok(())
}

fn parse_exp(expression: &str, memo_mode: &mut Option<Memorize>) -> Result<Expr, String> {
    match expression.to_lowercase().parse::<StackData>() {
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
            "n!" | "!" | "fact" | "factorial" => Ok(Expr::Monomial(MonomialFunc::Factorial)),
            "pi" => Ok(Expr::Const(Constant::Pi)),
            "e" => Ok(Expr::Const(Constant::E)),
            "sum" => Ok(Expr::Opstack(OperateStack::Sum)),
            "torad" => Ok(Expr::Monomial(MonomialFunc::ToRad)),
            "todeg" => Ok(Expr::Monomial(MonomialFunc::ToDeg)),
            "abs" => Ok(Expr::Monomial(MonomialFunc::Abs)),
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

#[cfg(test)]
mod tests {

    use crate::{manage_stack, DegMode, StackData};
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
            match &teststack[0] {
                StackData::Number(data) => (*data, 0.0),
                StackData::Complex(data) => (data.re, data.im),
            }
        };

        let realnumtest = |exp| {
            let (data, _): (f64, f64) = test_manage(exp);
            data
        };

        let complexnumtest = |exp| {
            let (re, im) = test_manage(exp);
            (re, im)
        };
        let complex_assert = |(re, im): (f64, f64), (re2, im2): (f64, f64)| {
            assert!((re - re2).abs() < 1e-10);
            assert!((im - im2).abs() < 1e-10);
        };

        assert_eq!(realnumtest("2 3 +"), 5.0);
        assert_eq!(realnumtest("2 3 -"), -1.0);
        assert_eq!(realnumtest("2 3 *"), 6.0);
        assert_eq!(realnumtest("3 2 /"), 1.5);
        assert_eq!(realnumtest("3 2 ^"), 9.0);
        assert_eq!(realnumtest("3 2/"), 1.5);
        assert_eq!(realnumtest("3 2/ 2 * 2^"), 9.0);
        assert_eq!(realnumtest("15 4 %"), 3.0);
        assert_eq!(realnumtest("51 19 %"), 13.0);
        assert_eq!(realnumtest("9.0 sqrt"), 3.0);
        assert_eq!(realnumtest("10.0 ln"), f64::consts::LN_10);
        assert_eq!(realnumtest("100.0 log"), 2.0);
        assert_eq!(realnumtest("9.0 3 3 5 sum"), 20.0);
        assert!((realnumtest("pi 6 / sin") - 0.5) < 1e-10);
        assert!((realnumtest("pi 3 / cos") - 0.5) < 1e-10);
        assert!((realnumtest("pi 4 / tan") - 1.0) < 1e-10);
        assert!((realnumtest("0.5 asin") - f64::consts::PI / 6.0) < 1e-10);
        assert!((realnumtest("0.5 acos") - f64::consts::PI / 3.0) < 1e-10);
        assert!((realnumtest("1.0 atan") - f64::consts::PI / 4.0) < 1e-10);
        assert_eq!(realnumtest("pi"), f64::consts::PI);
        assert_eq!(realnumtest("e"), f64::consts::E);
        assert!((realnumtest("60 torad") - f64::consts::PI / 3.0).abs() < 1e-10);
        assert!((realnumtest("pi 3 / todeg") - 60.0).abs() < 1e-10);
        assert_eq!(realnumtest("2 3 + 12 *"), 60.0);
        assert_eq!(realnumtest("3 3 3 sum sqrt"), 3.0);
        assert_eq!(realnumtest("10 3 npr"), 720.0);
        assert_eq!(realnumtest("10 3 ncr"), 120.0);
        assert_eq!(realnumtest("5 n!"), 120.0);
        complex_assert(complexnumtest("2+3i 3+4i +"), (5.0, 7.0));
        complex_assert(complexnumtest("2+3i 3+4i -"), (-1.0, -1.0));
        complex_assert(complexnumtest("2+3i 3+4i *"), (-6.0, 17.0));
        complex_assert(complexnumtest("2+3i 3+4i /"), (0.72, 0.04));
        complex_assert(complexnumtest("2+3i 2 ^"), (-5.0, 12.0));
        complex_assert(complexnumtest("2+3i 2 ^ 5 +"), (0.0, 12.0));
        complex_assert(complexnumtest("3+4i abs"), (5.0, 0.0));
        Ok(())
    }
}
