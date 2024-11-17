use core::f64;
use crossterm::execute;
use crossterm::terminal::{size, ClearType, ScrollUp, SetSize};
use rustyline::DefaultEditor;
use std::io;
enum Expr {
    Numbers(f64),
    Add,
    Subtract,
    Multiply,
    Divide,
    Pow,
    Sqrt,
    Log,
    Sum,
    Sin,
    Cos,
    Tan,
    ASin,
    ACos,
    ATan,
    Pi,
    E,
    Swap,
    Clear,
    ToRad,
    ToDeg,
    Deg,
    Rad,
}

#[derive(Debug)]
enum DegMode {
    Rad,
    Deg,
}

fn manage_stack(
    expression: &str,
    calstack: &mut Vec<f64>,
    degmode: &mut DegMode,
) -> Result<(), String> {
    let mut items = Vec::new();
    for exp in expression.split_whitespace() {
        let initem = parse_exp(exp)?;
        items.push(initem);
    }

    for item in items {
        match item {
            Expr::Numbers(data) => calstack.push(data),
            Expr::Add => {
                let (exex, ex) = get_two_item(calstack)?;
                calstack.push(exex + ex)
            }
            Expr::Subtract => {
                let (exex, ex) = get_two_item(calstack)?;
                calstack.push(exex - ex)
            }
            Expr::Multiply => {
                let (exex, ex) = get_two_item(calstack)?;
                calstack.push(exex * ex)
            }
            Expr::Divide => {
                let (exex, ex) = get_two_item(calstack)?;
                if ex == 0f64 {
                    return Err("Zero Divided Error".to_string());
                } else {
                    calstack.push(exex / ex)
                }
            }
            Expr::Pow => {
                let (exex, ex) = get_two_item(calstack)?;
                calstack.push(exex.powf(ex));
            }
            Expr::Sqrt => {
                let ex = get_one_item(calstack)?;
                calstack.push(ex.sqrt());
            }
            Expr::Log => {
                let (exex, ex) = get_two_item(calstack)?;
                calstack.push(exex.log(ex));
            }
            Expr::Sum => {
                let sum_result = calstack.iter().sum();
                calstack.clear();
                calstack.push(sum_result);
            }
            Expr::Sin => {
                let ex = to_rad_item(get_one_item(calstack)?, degmode);
                calstack.push(ex.sin())
            }
            Expr::Cos => {
                let ex = to_rad_item(get_one_item(calstack)?, degmode);
                calstack.push(ex.cos())
            }
            Expr::Tan => {
                let ex = to_rad_item(get_one_item(calstack)?, degmode);
                calstack.push(ex.tan())
            }
            Expr::ASin => {
                let ex = get_one_item(calstack)?;
                // asin -> radian
                calstack.push(to_deg_item(ex.asin(), degmode));
            }
            Expr::ACos => {
                let ex = get_one_item(calstack)?;
                calstack.push(to_deg_item(ex.acos(), degmode));
            }
            Expr::ATan => {
                let ex = get_one_item(calstack)?;
                calstack.push(to_deg_item(ex.atan(), degmode));
            }
            Expr::Pi => {
                calstack.push(f64::consts::PI);
            }
            Expr::E => {
                calstack.push(f64::consts::E);
            }
            Expr::ToDeg => {
                let ex = get_one_item(calstack)?;
                calstack.push(to_deg(ex));
            }
            Expr::ToRad => {
                let ex = get_one_item(calstack)?;
                calstack.push(to_rad(ex));
            }
            Expr::Swap => {
                if calstack.len() < 2 {
                    return Err("Stack is too short".to_string());
                } else {
                    let last = calstack.len() - 1;
                    calstack.swap(last, last - 1);
                }
            }
            Expr::Clear => calstack.clear(),
            Expr::Deg => *degmode = DegMode::Deg,
            Expr::Rad => *degmode = DegMode::Rad,
        }
    }
    Ok(())
}

fn parse_exp(expression: &str) -> Result<Expr, String> {
    match expression.to_lowercase().parse::<f64>() {
        Ok(data) => Ok(Expr::Numbers(data)),
        Err(_) => match expression {
            "+" => Ok(Expr::Add),
            "-" => Ok(Expr::Subtract),
            "*" => Ok(Expr::Multiply),
            "/" => Ok(Expr::Divide),
            "c" | "cl" | "clear" => Ok(Expr::Clear),
            "sw" | "swap" => Ok(Expr::Swap),
            "sin" => Ok(Expr::Sin),
            "cos" => Ok(Expr::Cos),
            "tan" => Ok(Expr::Tan),
            "asin" => Ok(Expr::ASin),
            "acos" => Ok(Expr::ACos),
            "atan" => Ok(Expr::ATan),
            "^" | "pow" => Ok(Expr::Pow),
            "sqrt" => Ok(Expr::Sqrt),
            "log" => Ok(Expr::Log),
            "pi" => Ok(Expr::Pi),
            "e" => Ok(Expr::E),
            "sum" => Ok(Expr::Sum),
            "torad" => Ok(Expr::ToRad),
            "todeg" => Ok(Expr::ToDeg),
            "rad" => Ok(Expr::Rad),
            "deg" => Ok(Expr::Deg),
            _ => Err("Invalid Data".to_string()),
        },
    }
}

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
fn to_rad(val: f64) -> f64 {
    val / 180.0 * f64::consts::PI
}
fn to_deg(val: f64) -> f64 {
    val / f64::consts::PI * 180.0
}
fn to_rad_item(val: f64, mode: &DegMode) -> f64 {
    match mode {
        DegMode::Rad => val,
        DegMode::Deg => to_rad(val),
    }
}
fn to_deg_item(val: f64, mode: &DegMode) -> f64 {
    match mode {
        DegMode::Rad => val,
        DegMode::Deg => to_deg(val),
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (cols, rows) = size()?;
    // Resize terminal and scroll up.
    execute!(io::stdout(), SetSize(60, 20), ScrollUp(15))?;
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
        assert!((test_manage("pi 6 / sin") - 0.5).abs() < 1e-10);
        assert!((test_manage("pi 3 / cos") - 0.5).abs() < 1e-10);
        assert!((test_manage("pi 4 / tan") - 1.0).abs() < 1e-10);
        assert!((test_manage("0.5 asin") - f64::consts::PI / 6.0).abs() < 1e-10);
        assert!((test_manage("0.5 acos") - f64::consts::PI / 3.0).abs() < 1e-10);
        assert!((test_manage("1.0 atan") - f64::consts::PI / 4.0).abs() < 1e-10);
        assert_eq!(test_manage("pi"), f64::consts::PI);
        assert_eq!(test_manage("e"), f64::consts::E);
        assert_eq!(test_manage("60 torad"), f64::consts::PI / 3.0);
        assert_eq!(test_manage("pi 3 / todeg"), 60.0);
        assert_eq!(test_manage("2 3 + 12 *"), 60.0);
        assert_eq!(test_manage("3 3 3 sum sqrt"), 3.0);
        Ok(())
    }
}
