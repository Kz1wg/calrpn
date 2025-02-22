use core::f64;
use num::complex::Complex;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::ops::{Add, Div, Mul, Rem, Sub};
use std::str::FromStr;
use std::vec;

trait Help {
    fn help(&self) -> &str;
    fn show_help() -> String;
}
// 演算時要素の列挙型
#[derive(Debug)]
pub enum Expr {
    Numbers(CalcNum),
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
pub fn print_help() {
    let allhelp = vec![
        BinomialFunc::show_help(),
        MonomialFunc::show_help(),
        Constant::show_help(),
        OperateStack::show_help(),
        DegMode::show_help(),
        Memorize::show_help(),
    ];
    println!("calrpn");
    for help in allhelp {
        println!("{}", help);
    }
}
impl Help for BinomialFunc {
    fn help(&self) -> &str {
        match self {
            BinomialFunc::Add => "1 2 + : 1 + 2",
            BinomialFunc::Subtract => "1 2 - : 1 - 2",
            BinomialFunc::Multiply => "1 2 * : 1 * 2",
            BinomialFunc::Divide => "1 2 / : 1 / 2",
            BinomialFunc::Mod => "1 2 % : 1 % 2",
            BinomialFunc::Pow => "3 2 ^ : 3 ^ 2",
            BinomialFunc::NCr => "10 2 ncr : 10 nCr 2",
            BinomialFunc::NPr => "10 2 npr : 10 nPr 2",
        }
    }
    fn show_help() -> String {
        [
            BinomialFunc::Add,
            BinomialFunc::Subtract,
            BinomialFunc::Multiply,
            BinomialFunc::Divide,
            BinomialFunc::Mod,
            BinomialFunc::Pow,
            BinomialFunc::NCr,
            BinomialFunc::NPr,
        ]
        .map(|x| x.help().to_string())
        .join("\n")
    }
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
    ToPolar,
    ToRec,
}
impl Help for MonomialFunc {
    fn help(&self) -> &str {
        match self {
            MonomialFunc::Sqrt => "30 sqrt : sqrt(30)",
            MonomialFunc::Log => "30 log : log10(30)",
            MonomialFunc::Ln => "30 ln : ln(30)",
            MonomialFunc::Sin => "30 sin : sin(30)",
            MonomialFunc::Cos => "30 cos : cos(30)",
            MonomialFunc::Tan => "30 tan : tan(30)",
            MonomialFunc::ASin => "30 asin : asin(30)",
            MonomialFunc::ACos => "30 acos : acos(30)",
            MonomialFunc::ATan => "30 atan : atan(30)",
            MonomialFunc::ToDeg => "pi todeg : pi to degrees",
            MonomialFunc::ToRad => "30 torad : 30 to radians",
            MonomialFunc::Abs => "30 abs : abs(30)",
            MonomialFunc::Factorial => "10 ! : factorial(10)",
            MonomialFunc::ToPolar => "30+2i topolar : 30+2i to polar",
            MonomialFunc::ToRec => "30+45i torec : 30+45i to rectangular",
        }
    }
    fn show_help() -> String {
        [
            MonomialFunc::Sqrt,
            MonomialFunc::Log,
            MonomialFunc::Ln,
            MonomialFunc::Sin,
            MonomialFunc::Cos,
            MonomialFunc::Tan,
            MonomialFunc::ASin,
            MonomialFunc::ACos,
            MonomialFunc::ATan,
            MonomialFunc::ToDeg,
            MonomialFunc::ToRad,
            MonomialFunc::Abs,
            MonomialFunc::Factorial,
            MonomialFunc::ToPolar,
            MonomialFunc::ToRec,
        ]
        .map(|x| x.help().to_string())
        .join("\n")
    }
}
// 記憶領域の列挙型
#[derive(Debug)]
pub enum Memorize {
    Recall(Option<String>),
    Clear,
    Store(Option<String>),
}
impl Help for Memorize {
    fn help(&self) -> &str {
        match self {
            Memorize::Recall(_) => "rcl : recall memory",
            Memorize::Clear => "mc : clear memory",
            Memorize::Store(_) => "sto : store memory",
        }
    }
    fn show_help() -> String {
        [
            Memorize::Recall(None),
            Memorize::Clear,
            Memorize::Store(None),
        ]
        .map(|x| x.help().to_string())
        .join("\n")
    }
}
// 定数の列挙型
#[derive(Debug)]
pub enum Constant {
    Pi,
    E,
}
impl Help for Constant {
    fn help(&self) -> &str {
        match self {
            Constant::Pi => "pi : 3.14159265358979323846",
            Constant::E => "e : 2.71828182845904523536",
        }
    }
    fn show_help() -> String {
        [Constant::Pi, Constant::E]
            .map(|x| x.help().to_string())
            .join("\n")
    }
}

// Stack & Mode操作の列挙型
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
impl Help for OperateStack {
    fn help(&self) -> &str {
        match self {
            OperateStack::Swap => "sw : swap top two stack",
            OperateStack::Clear => "cl : clear stack",
            OperateStack::Delete => "dl : delete top stack",
            OperateStack::RollUp => "rup : roll up stack",
            OperateStack::RollDown => "rdn : roll down stack",
            OperateStack::Sum => "sum : sum all stack",
            OperateStack::Deg => "deg : set degree mode",
            OperateStack::Rad => "rad : set radian mode",
        }
    }
    fn show_help() -> String {
        [
            OperateStack::Swap,
            OperateStack::Clear,
            OperateStack::Delete,
            OperateStack::RollUp,
            OperateStack::RollDown,
            OperateStack::Sum,
            OperateStack::Deg,
            OperateStack::Rad,
        ]
        .map(|x| x.help().to_string())
        .join("\n")
    }
}
// 角度モードの列挙型
#[derive(Debug)]
pub enum DegMode {
    Rad,
    Deg,
}
impl Help for DegMode {
    fn help(&self) -> &str {
        match self {
            DegMode::Rad => "Rad : Radian Mode",
            DegMode::Deg => "Deg : Degree Mode",
        }
    }
    fn show_help() -> String {
        [DegMode::Rad, DegMode::Deg]
            .map(|x| x.help().to_string())
            .join("\n")
    }
}

#[derive(Debug, Clone)]
pub enum CalcNum {
    Number(f64),
    Complex(Complex<f64>),
}
impl FromStr for CalcNum {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<f64>() {
            Ok(val) => Ok(CalcNum::Number(val)),
            Err(_) => match s.parse::<Complex<f64>>() {
                Ok(val) => Ok(CalcNum::Complex(val)),
                Err(_) => Err("Parse Error".to_string()),
            },
        }
    }
}

impl Add for CalcNum {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        match (self, other) {
            (CalcNum::Number(a), CalcNum::Number(b)) => CalcNum::Number(a + b),
            (CalcNum::Complex(a), CalcNum::Complex(b)) => CalcNum::Complex(a + b),
            (CalcNum::Number(a), CalcNum::Complex(b)) => CalcNum::Complex(Complex::new(a, 0.0) + b),
            (CalcNum::Complex(a), CalcNum::Number(b)) => CalcNum::Complex(a + Complex::new(b, 0.0)),
        }
    }
}
impl Sub for CalcNum {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (CalcNum::Number(a), CalcNum::Number(b)) => CalcNum::Number(a - b),
            (CalcNum::Complex(a), CalcNum::Complex(b)) => CalcNum::Complex(a - b),
            (CalcNum::Number(a), CalcNum::Complex(b)) => CalcNum::Complex(Complex::new(a, 0.0) - b),
            (CalcNum::Complex(a), CalcNum::Number(b)) => CalcNum::Complex(a - Complex::new(b, 0.0)),
        }
    }
}
impl Mul for CalcNum {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (CalcNum::Number(a), CalcNum::Number(b)) => CalcNum::Number(a * b),
            (CalcNum::Complex(a), CalcNum::Complex(b)) => CalcNum::Complex(a * b),
            (CalcNum::Number(a), CalcNum::Complex(b)) => CalcNum::Complex(Complex::new(a, 0.0) * b),
            (CalcNum::Complex(a), CalcNum::Number(b)) => CalcNum::Complex(a * Complex::new(b, 0.0)),
        }
    }
}

impl Div for CalcNum {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        match (self, other) {
            (CalcNum::Number(a), CalcNum::Number(b)) => CalcNum::Number(a / b),
            (CalcNum::Complex(a), CalcNum::Complex(b)) => CalcNum::Complex(a / b),
            (CalcNum::Number(a), CalcNum::Complex(b)) => CalcNum::Complex(Complex::new(a, 0.0) / b),
            (CalcNum::Complex(a), CalcNum::Number(b)) => CalcNum::Complex(a / Complex::new(b, 0.0)),
        }
    }
}
impl Rem for CalcNum {
    type Output = Self;
    fn rem(self, other: Self) -> Self {
        match (self, other) {
            (CalcNum::Number(a), CalcNum::Number(b)) => CalcNum::Number(a % b),
            (CalcNum::Complex(a), CalcNum::Complex(b)) => CalcNum::Complex(a % b),
            (CalcNum::Number(a), CalcNum::Complex(b)) => CalcNum::Complex(Complex::new(a, 0.0) % b),
            (CalcNum::Complex(a), CalcNum::Number(b)) => CalcNum::Complex(a % Complex::new(b, 0.0)),
        }
    }
}
impl CalcNum {
    pub fn num_format(&self, n_place: usize) -> String {
        match self {
            CalcNum::Number(val) => match self.is_integer() {
                true => format!("{:.0}", val),
                false => format!("{:.1$}", val, n_place),
            },
            CalcNum::Complex(val) => {
                format!("{:.2$}  i:{:.2$}", val.re, val.im, n_place)
            }
        }
    }

    fn is_realnumber(&self) -> bool {
        matches!(self, CalcNum::Number(_))
    }

    fn is_integer(&self) -> bool {
        match self {
            CalcNum::Number(val) => val.fract() == 0.0,
            _ => false,
        }
    }

    fn get_realnumber(&self) -> Result<f64, Box<dyn std::error::Error>> {
        match self {
            CalcNum::Number(val) => Ok(*val),
            CalcNum::Complex(_val) => Err("Complex number is not supported".into()),
        }
    }
    fn pow(&self, n: &Self) -> CalcNum {
        match (self, n) {
            (CalcNum::Number(val), CalcNum::Number(n)) => CalcNum::Number(val.powf(*n)),
            (CalcNum::Complex(val), CalcNum::Number(n)) => CalcNum::Complex(val.powf(*n)),
            (CalcNum::Number(val), CalcNum::Complex(n)) => {
                CalcNum::Complex(Complex::new(*val, 0.0).powc(*n))
            }
            (CalcNum::Complex(val), CalcNum::Complex(n)) => CalcNum::Complex(val.powc(*n)),
        }
    }
    fn log10(&self) -> CalcNum {
        match self {
            CalcNum::Number(val) => CalcNum::Number(val.log10()),
            CalcNum::Complex(val) => CalcNum::Complex(val.log10()),
        }
    }
    fn ln(&self) -> CalcNum {
        match self {
            CalcNum::Number(val) => CalcNum::Number(val.ln()),
            CalcNum::Complex(val) => CalcNum::Complex(val.ln()),
        }
    }

    fn sqrt(&self) -> CalcNum {
        match self {
            CalcNum::Number(val) => {
                if *val < 0.0 {
                    CalcNum::Complex(Complex::new(0.0, val.abs().sqrt()))
                } else {
                    CalcNum::Number(val.sqrt())
                }
            }
            CalcNum::Complex(val) => CalcNum::Complex(val.sqrt()),
        }
    }

    fn sin(&self, degmode: &DegMode) -> CalcNum {
        match self {
            CalcNum::Number(val) => CalcNum::Number(match degmode {
                DegMode::Deg => val.to_radians().sin(),
                DegMode::Rad => val.sin(),
            }),
            CalcNum::Complex(val) => CalcNum::Complex(val.sin()),
        }
    }
    fn cos(&self, degmode: &DegMode) -> CalcNum {
        match self {
            CalcNum::Number(val) => CalcNum::Number(match degmode {
                DegMode::Deg => val.to_radians().cos(),
                DegMode::Rad => val.cos(),
            }),
            CalcNum::Complex(val) => CalcNum::Complex(val.cos()),
        }
    }
    fn tan(&self, degmode: &DegMode) -> CalcNum {
        match self {
            CalcNum::Number(val) => CalcNum::Number(match degmode {
                DegMode::Deg => val.to_radians().tan(),
                DegMode::Rad => val.tan(),
            }),
            CalcNum::Complex(val) => CalcNum::Complex(val.tan()),
        }
    }
    fn asin(&self, degmode: &DegMode) -> CalcNum {
        match self {
            CalcNum::Number(val) => CalcNum::Number(match degmode {
                DegMode::Deg => val.asin().to_degrees(),
                DegMode::Rad => val.asin(),
            }),
            CalcNum::Complex(val) => CalcNum::Complex(val.asin()),
        }
    }
    fn acos(&self, degmode: &DegMode) -> CalcNum {
        match self {
            CalcNum::Number(val) => CalcNum::Number(match degmode {
                DegMode::Deg => val.acos().to_degrees(),
                DegMode::Rad => val.acos(),
            }),
            CalcNum::Complex(val) => CalcNum::Complex(val.acos()),
        }
    }
    fn atan(&self, degmode: &DegMode) -> CalcNum {
        match self {
            CalcNum::Number(val) => CalcNum::Number(match degmode {
                DegMode::Deg => val.atan().to_degrees(),
                DegMode::Rad => val.atan(),
            }),
            CalcNum::Complex(val) => CalcNum::Complex(val.atan()),
        }
    }

    fn to_polar(&self, degmode: &DegMode) -> Result<CalcNum, Box<dyn std::error::Error>> {
        match self {
            CalcNum::Number(_val) => Err("Invalid Type".into()),
            CalcNum::Complex(val) => {
                let result = val.to_polar();
                let angle = match degmode {
                    DegMode::Deg => CalcNum::Number(result.1).to_deg()?,
                    DegMode::Rad => CalcNum::Number(result.1),
                };
                Ok(CalcNum::Complex(Complex {
                    re: result.0,
                    im: angle.get_realnumber()?,
                }))
            }
        }
    }
    fn to_rectangular(&self, degmode: &DegMode) -> Result<CalcNum, Box<dyn std::error::Error>> {
        match self {
            CalcNum::Complex(polardata) => {
                let theta = match degmode {
                    DegMode::Deg => polardata.im.to_radians(),
                    DegMode::Rad => polardata.im,
                };
                Ok(CalcNum::Complex(Complex::from_polar(polardata.re, theta)))
            }
            CalcNum::Number(_) => Err("Invalid Type".into()),
        }
    }

    fn to_deg(&self) -> Result<CalcNum, Box<dyn std::error::Error>> {
        match self {
            CalcNum::Number(val) => Ok(CalcNum::Number(val.to_degrees())),
            CalcNum::Complex(_val) => Err("Complex number is not supported".into()),
        }
    }

    fn to_rad(&self) -> Result<CalcNum, Box<dyn std::error::Error>> {
        match self {
            CalcNum::Number(val) => Ok(CalcNum::Number(val.to_radians())),
            CalcNum::Complex(_val) => Err("Complex number is not supported".into()),
        }
    }

    fn factorial(&self) -> Result<CalcNum, Box<dyn std::error::Error>> {
        // 階乗計算
        if !self.is_integer() {
            return Err("Factorial is only supported for integer".into());
        }
        match self {
            CalcNum::Number(val) => Ok(CalcNum::Number((1..=*val as u64).product::<u64>() as f64)),
            CalcNum::Complex(_val) => Err("Complex number is not supported".into()),
        }
    }

    fn permutation(&self, n: &Self) -> Result<CalcNum, Box<dyn std::error::Error>> {
        // 順列計算
        if !self.is_integer() || !n.is_integer() {
            return Err("Permutation is only supported for integer".into());
        }
        match self {
            CalcNum::Number(val) => Ok(CalcNum::Number(
                (1..=*val as u64)
                    .rev()
                    .take(n.get_realnumber()? as usize)
                    .product::<u64>() as f64,
            )),
            CalcNum::Complex(_val) => Err("Complex number is not supported".into()),
        }
    }

    fn combination(&self, n: &Self) -> Result<CalcNum, Box<dyn std::error::Error>> {
        // 組み合わせ計算
        if !self.is_integer() || !n.is_integer() {
            return Err("Combination is only supported for integer".into());
        }
        match self {
            CalcNum::Number(val) => Ok(CalcNum::Number(
                (1..=*val as u64)
                    .rev()
                    .take(n.get_realnumber()? as usize)
                    .product::<u64>() as f64
                    / (1..=n.get_realnumber()? as u64).product::<u64>() as f64,
            )),
            CalcNum::Complex(_val) => Err("Complex number is not supported".into()),
        }
    }

    fn abs(&self) -> CalcNum {
        match self {
            CalcNum::Number(val) => CalcNum::Number(val.abs()),
            CalcNum::Complex(val) => CalcNum::Number(val.norm()),
        }
    }
}

pub const STACK_SIZE: usize = 12;

// スタックの管理関数
pub fn manage_stack(
    expression: &str,
    calstack: &mut VecDeque<CalcNum>,
    degmode: &mut DegMode,
    memory_map: &mut BTreeMap<String, CalcNum>,
    memo_mode: &mut Option<Memorize>,
) -> Result<(), Box<dyn std::error::Error>> {
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

    // 記憶関連の処理
    let mut manage_memorize = |memo: Memorize, calstack: &mut VecDeque<CalcNum>| {
        match memo {
            Memorize::Recall(key) => {
                if let Some(inkey) = key {
                    if let Some(val) = memory_map.get(&inkey) {
                        calstack.push_back(val.clone());
                    } else {
                        return Err("No Key");
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
                        return Err("Stack is Empty");
                    }
                }
            }
        }
        Ok(())
    };

    // 二項演算の処理
    let manage_binomial = |b_func: BinomialFunc, calstack: &mut VecDeque<CalcNum>| {
        let (exex, ex) = get_two_item(calstack)?;
        let result = match b_func {
            BinomialFunc::Add => exex + ex,
            BinomialFunc::Subtract => exex - ex,
            BinomialFunc::Multiply => exex * ex,
            BinomialFunc::Divide => exex / ex,
            BinomialFunc::Mod => exex % ex,
            BinomialFunc::Pow => exex.pow(&ex),
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
        Ok(())
    };

    // 単項演算の処理
    let manage_monomial = |m_func, calstack: &mut VecDeque<CalcNum>, degmode: &mut DegMode| {
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
            MonomialFunc::ToDeg => match ex.to_deg() {
                Ok(deg) => deg,
                Err(e) => {
                    calstack.push_back(ex);
                    return Err(e);
                }
            },
            MonomialFunc::ToRad => match ex.to_rad() {
                Ok(rad) => rad,
                Err(e) => {
                    calstack.push_back(ex);
                    return Err(e);
                }
            },
            MonomialFunc::Factorial => match ex.factorial() {
                Ok(result) => result,
                Err(e) => {
                    calstack.push_back(ex);
                    return Err(e);
                }
            },
            MonomialFunc::Abs => ex.abs(),
            MonomialFunc::ToPolar => match ex.to_polar(degmode) {
                Ok(polardata) => polardata,
                Err(e) => {
                    calstack.push_back(ex);
                    return Err(e);
                }
            },
            MonomialFunc::ToRec => match ex.to_rectangular(degmode) {
                Ok(result) => result,
                Err(e) => {
                    calstack.push_back(ex);
                    return Err(e);
                }
            },
        };
        calstack.push_back(result);
        Ok(())
    };

    // スタック操作・演算の処理
    let manage_operate_stack =
        |operate, calstack: &mut VecDeque<CalcNum>, degmode: &mut DegMode| {
            match operate {
                OperateStack::Swap => {
                    if calstack.len() < 2 {
                        return Err("Stack is too short");
                    } else {
                        let last = calstack.len() - 1;
                        calstack.swap(last, last - 1);
                    }
                }
                OperateStack::Clear => calstack.clear(),
                OperateStack::Delete => {
                    if calstack.is_empty() {
                        return Err("Stack is Empty");
                    } else {
                        calstack.pop_back();
                    }
                }
                OperateStack::RollUp => {
                    if calstack.len() < 2 {
                        return Err("Stack is too short");
                    } else {
                        let last = calstack.pop_back().unwrap();
                        calstack.push_front(last);
                    }
                }
                OperateStack::RollDown => {
                    if calstack.len() < 2 {
                        return Err("Stack is too short");
                    } else {
                        let first = calstack.pop_front().unwrap();
                        calstack.push_back(first);
                    }
                }
                OperateStack::Sum => {
                    if calstack.iter().all(CalcNum::is_realnumber) {
                        let sum_result = calstack.iter().map(|x| x.get_realnumber().unwrap()).sum();
                        calstack.clear();
                        calstack.push_back(CalcNum::Number(sum_result));
                    } else {
                        return Err("Invalid Data");
                    }
                }
                OperateStack::Deg => *degmode = DegMode::Deg,
                OperateStack::Rad => *degmode = DegMode::Rad,
            }
            Ok(())
        };

    // 定数の処理
    let manage_constant = |consts, calstack: &mut VecDeque<CalcNum>| {
        let result = match consts {
            Constant::Pi => f64::consts::PI,
            Constant::E => f64::consts::E,
        };
        calstack.push_back(CalcNum::Number(result));
    };

    // 式の要素を順番に処理
    for item in items {
        // 式の要素に応じて処理を分岐
        match item {
            Expr::Memo(mem) => manage_memorize(mem, calstack)?,
            Expr::Numbers(data) => calstack.push_back(data),
            Expr::Binomial(b_func) => manage_binomial(b_func, calstack)?,
            Expr::Monomial(m_func) => manage_monomial(m_func, calstack, degmode)?,
            Expr::Opstack(operate) => manage_operate_stack(operate, calstack, degmode)?,
            Expr::Const(consts) => manage_constant(consts, calstack),
        }
    }
    // スタックが一定以上になった場合、先頭の要素を削除
    if calstack.len() >= STACK_SIZE {
        calstack.pop_front();
    };
    Ok(())
}

fn parse_exp(expression: &str, memo_mode: &mut Option<Memorize>) -> Result<Expr, String> {
    match expression.to_lowercase().parse::<CalcNum>() {
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
            "torec" | "torect" | "rec" | "rect" => Ok(Expr::Monomial(MonomialFunc::ToRec)),
            "topol" | "topolar" | "polar" | "pol" => Ok(Expr::Monomial(MonomialFunc::ToPolar)),
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
                    _ => Ok(Expr::Memo(Memorize::Recall(Some(expression.to_string())))),
                },
            },
        },
    }
}

// スタックから2つの要素を取り出す
fn get_two_item(calstack: &mut VecDeque<CalcNum>) -> Result<(CalcNum, CalcNum), String> {
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
fn get_one_item(calstack: &mut VecDeque<CalcNum>) -> Result<CalcNum, String> {
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

    use crate::{manage_stack, CalcNum, DegMode};
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
                CalcNum::Number(data) => (*data, 0.0),
                CalcNum::Complex(data) => (data.re, data.im),
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
        complex_assert(complexnumtest("-10i log"), (1.0, -0.6821881769));
        complex_assert(complexnumtest("-10i ln"), (2.302585093, -1.5707963268));
        complex_assert(complexnumtest("pi 1i*  sin"), (0.0, 11.5487393573));
        complex_assert(
            complexnumtest("pi 3 /  pi -4i / + cos"),
            (0.6623045446, -0.7522911202),
        );
        complex_assert(complexnumtest("-9 sqrt"), (0.0, 3.0));
        complex_assert(complexnumtest("2 3+4i +"), (5.0, 4.0));
        complex_assert(complexnumtest("2+3i 3+4i +"), (5.0, 7.0));
        complex_assert(complexnumtest("2+3i 3+4i +"), (5.0, 7.0));
        complex_assert(complexnumtest("2+3i 3+4i -"), (-1.0, -1.0));
        complex_assert(complexnumtest("2+3i 3+4i *"), (-6.0, 17.0));
        complex_assert(complexnumtest("2+3i 3+4i /"), (0.72, 0.04));
        complex_assert(complexnumtest("2+3i 2 ^"), (-5.0, 12.0));
        complex_assert(complexnumtest("2+3i 2 ^ 5 +"), (0.0, 12.0));
        complex_assert(complexnumtest("3+4i abs"), (5.0, 0.0));
        complex_assert(
            complexnumtest("e pi 3 / i * ^"),
            complexnumtest("pi 3 / cos pi 3 / sin i * +"),
        );
        complex_assert(complexnumtest("2+3i 2^"), (-5.0, 12.0));
        complex_assert(complexnumtest("i i ^"), complexnumtest("e pi -2 / ^"));
        complex_assert(complexnumtest("i i ^"), complexnumtest("e pi -2 / ^"));
        complex_assert(
            complexnumtest("1 pi 6 / i * + torec"),
            complexnumtest("pi 6 / cos pi 6 / sin i * +"),
        );
        complex_assert(
            complexnumtest("pi 6 / cos pi 6 / sin i * + topolar"),
            complexnumtest("1 pi 6 / i * + "),
        );

        Ok(())
    }
}
