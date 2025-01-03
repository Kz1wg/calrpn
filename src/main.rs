mod calcenums;
use calcenums::{BinomialFunc, Constant, DegMode, Expr, MonomialFunc, OperateStack};

use core::f64;
use crossterm::execute;
use crossterm::terminal::{size, ClearType, ScrollUp, SetSize};

use rustyline::DefaultEditor;
use std::io;

// スタックの管理関数
fn manage_stack(
    expression: &str,
    calstack: &mut Vec<f64>,
    degmode: &mut DegMode,
) -> Result<(), String> {
    // 入力された式を空白で分割し、それぞれの要素をparse_exp関数で処理
    let items = expression
        .split_whitespace()
        .map(parse_exp)
        .collect::<Result<Vec<Expr>, String>>()?;

    // 式の要素を順番に処理
    for item in items {
        // 式の要素に応じて処理を分岐
        match item {
            // 数値の場合
            Expr::Numbers(data) => calstack.push(data),
            // 二項演算の場合
            Expr::Binomial(b_func) => {
                let (exex, ex) = get_two_item(calstack)?;
                let result = match b_func {
                    BinomialFunc::Add => exex + ex,
                    BinomialFunc::Subtract => exex - ex,
                    BinomialFunc::Multiply => exex * ex,
                    BinomialFunc::Divide => {
                        if ex == 0f64 {
                            return Err("Zero Divided Error".to_string());
                        } else {
                            exex / ex
                        }
                    }
                    BinomialFunc::Pow => exex.powf(ex),
                    BinomialFunc::Log => exex.log(ex),
                };
                calstack.push(result);
            }
            // 単項演算の場合
            Expr::Monomial(m_func) => {
                let ex = get_one_item(calstack)?;
                let result = match m_func {
                    MonomialFunc::Sqrt => ex.sqrt(),
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
                };
                calstack.push(result);
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
                OperateStack::Sum => {
                    let sum_result = calstack.iter().sum();
                    calstack.clear();
                    calstack.push(sum_result);
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
                calstack.push(result);
            }
        }
    }
    Ok(())
}

fn parse_exp(expression: &str) -> Result<Expr, String> {
    match expression.to_lowercase().parse::<f64>() {
        Ok(data) => Ok(Expr::Numbers(data)),
        Err(_) => match expression {
            "+" => Ok(Expr::Binomial(BinomialFunc::Add)),
            "-" => Ok(Expr::Binomial(BinomialFunc::Subtract)),
            "*" => Ok(Expr::Binomial(BinomialFunc::Multiply)),
            "/" => Ok(Expr::Binomial(BinomialFunc::Divide)),
            "c" | "cl" | "clear" => Ok(Expr::Opstack(OperateStack::Clear)),
            "sw" | "swap" => Ok(Expr::Opstack(OperateStack::Swap)),
            "sin" => Ok(Expr::Monomial(MonomialFunc::Sin)),
            "cos" => Ok(Expr::Monomial(MonomialFunc::Cos)),
            "tan" => Ok(Expr::Monomial(MonomialFunc::Tan)),
            "asin" => Ok(Expr::Monomial(MonomialFunc::ASin)),
            "acos" => Ok(Expr::Monomial(MonomialFunc::ACos)),
            "atan" => Ok(Expr::Monomial(MonomialFunc::ATan)),
            "^" | "pow" => Ok(Expr::Binomial(BinomialFunc::Pow)),
            "sqrt" => Ok(Expr::Monomial(MonomialFunc::Sqrt)),
            "log" => Ok(Expr::Binomial(BinomialFunc::Log)),
            "pi" => Ok(Expr::Const(Constant::Pi)),
            "e" => Ok(Expr::Const(Constant::E)),
            "sum" => Ok(Expr::Opstack(OperateStack::Sum)),
            "torad" => Ok(Expr::Monomial(MonomialFunc::ToRad)),
            "todeg" => Ok(Expr::Monomial(MonomialFunc::ToDeg)),
            "rad" => Ok(Expr::Opstack(OperateStack::Rad)),
            "deg" => Ok(Expr::Opstack(OperateStack::Deg)),
            _ => Err("Invalid Data".to_string()),
        },
    }
}

// スタックから2つの要素を取り出す
fn get_two_item(calstack: &mut Vec<f64>) -> Result<(f64, f64), String> {
    if calstack.len() < 2 {
        Err("Stack is too short".to_string())
    } else {
        match (calstack.pop(), calstack.pop()) {
            (Some(ex), Some(exex)) => Ok((exex, ex)),
            _ => Err("Stack two pop Error".to_string()),
        }
    }
}
// スタックから1つの要素を取り出す
fn get_one_item(calstack: &mut Vec<f64>) -> Result<f64, String> {
    if calstack.is_empty() {
        Err("Stack is Empty".to_string())
    } else {
        match calstack.pop() {
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // プログラム終了後元のサイズに戻すため、元サイズを保持
    let (cols, rows) = size()?;
    // Resize terminal and scroll up.
    execute!(io::stdout(), SetSize(30, 18), ScrollUp(0))?;

    let mut stackdata: Vec<f64> = Vec::new();
    let mut rl = DefaultEditor::new()?;
    let mut degmode = DegMode::Deg;
    let clear = crossterm::terminal::Clear(ClearType::All);
    execute!(io::stdout(), clear)?;
    println!("Calrpn --Rpn calcurator--");

    loop {
        let readline = rl.readline(">> ");
        execute!(io::stdout(), clear)?;
        let mut message = String::new();

        if let Ok(data) = readline {
            if &data == "q" {
                break;
            };
            match manage_stack(&data, &mut stackdata, &mut degmode) {
                Ok(()) => (),
                Err(e) => message = e,
            }
        }

        println!("[{:?}]", degmode);

        for item in stackdata.iter() {
            println!("{:.3}", item)
        }

        if !message.is_empty() {
            eprintln!("{message}");
        }
    }
    execute!(io::stdout(), SetSize(cols, rows))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use core::f64;

    use crate::{manage_stack, DegMode};

    #[test]
    fn test() -> Result<(), String> {
        let test_manage = |exp| {
            let mut teststack = Vec::new();
            match manage_stack(exp, &mut teststack, &mut DegMode::Rad) {
                Ok(_) => (),
                Err(e) => eprintln!("{e}"),
            };
            teststack[0]
        };
        assert_eq!(test_manage("2 3 +"), 5.0);
        assert_eq!(test_manage("2 3 -"), -1.0);
        assert_eq!(test_manage("2 3 *"), 6.0);
        assert_eq!(test_manage("3 2 /"), 1.5);
        assert_eq!(test_manage("3 2 ^"), 9.0);
        assert_eq!(test_manage("9.0 sqrt"), 3.0);
        assert_eq!(test_manage("9.0 3 log"), 2.0);
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
        Ok(())
    }
}
