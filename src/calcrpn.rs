use crate::commands;
use crate::finance;
use num::complex::Complex;
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::f64::consts;
use std::ops::{Add, Div, Mul, Rem, Sub};
use std::str::FromStr;

type CalcResult<T> = Result<T, Box<dyn std::error::Error>>;

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
#[derive(Debug, Clone, Copy)]
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
#[derive(Debug, Clone, Copy)]
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
    Chs,
    Inv,
    Square,
    Exp,
    Pow10,
    Round,
}

// 記憶領域の列挙型
// キーが None のものは「キー待ち」状態を表し、次のトークンをキーとして受け取る
#[derive(Debug, Clone)]
pub enum Memorize {
    Recall(Option<String>),
    Clear,
    Delete(Option<String>),
    Store(Option<String>),
    Tvm(Option<String>),
}

impl Memorize {
    fn is_waiting_key(&self) -> bool {
        matches!(
            self,
            Memorize::Recall(None)
                | Memorize::Store(None)
                | Memorize::Delete(None)
                | Memorize::Tvm(None)
        )
    }
}

// 定数の列挙型
#[derive(Debug, Clone, Copy)]
pub enum Constant {
    Pi,
    E,
}

impl Constant {
    fn value(self) -> CalcNum {
        match self {
            Constant::Pi => CalcNum::Number(consts::PI),
            Constant::E => CalcNum::Number(consts::E),
        }
    }
}

// Stack & Mode操作の列挙型
#[derive(Debug, Clone, Copy)]
pub enum OperateStack {
    Swap,
    Clear,
    Delete,
    RollUp,
    RollDown,
    Sum,
    LastX,
    Deg,
    Rad,
}

// 角度モードの列挙型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DegMode {
    Rad,
    Deg,
}

#[derive(Debug, Clone, PartialEq)]
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
                Err(e) => Err(e.to_string()),
            },
        }
    }
}

// 演算子のオーバーライド
impl Add for CalcNum {
    // 加算
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
    // 引き算
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
    // 乗算
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
    // 除算
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
    // 余算
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

// 数値の表記法
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Notation {
    Fix, // 固定小数点 (極端な値は自動で指数表記)
    Sci, // 指数表記 1.235e+03
    Eng, // 工学表記 (指数が3の倍数) 12.345e+03
}

impl Notation {
    pub fn label(self) -> &'static str {
        match self {
            Notation::Fix => "FIX",
            Notation::Sci => "SCI",
            Notation::Eng => "ENG",
        }
    }
}

// 数値の表示形式
#[derive(Debug, Clone, Copy)]
pub struct DisplayFormat {
    pub notation: Notation,
    pub decimals: usize,
    // 3桁区切り (FIX のみ)
    pub grouping: bool,
}

impl Default for DisplayFormat {
    fn default() -> Self {
        Self {
            notation: Notation::Fix,
            decimals: 3,
            grouping: true,
        }
    }
}

// FIX でこれ以上大きい値は指数表記にする (f64 で整数を正確に表せる範囲の目安)
const FIX_MAX: f64 = 1e15;

// 実数を整形する。trim_integer が true なら整数は小数部を省く
fn format_real(v: f64, fmt: &DisplayFormat, trim_integer: bool) -> String {
    if !v.is_finite() {
        return format!("{v}"); // inf, -inf, NaN
    }
    // -0 を 0 として表示
    let v = if v == 0.0 { 0.0 } else { v };
    let n = fmt.decimals;
    match fmt.notation {
        Notation::Sci => format_sci(v, n),
        Notation::Eng => format_eng(v, n),
        Notation::Fix => {
            let abs = v.abs();
            // 大きすぎる値、または小さすぎて 0 に見えてしまう値は指数表記にする
            let too_small = abs != 0.0 && abs < 0.5 * 10f64.powi(-(n as i32));
            if abs >= FIX_MAX || too_small {
                return format_sci(v, n);
            }
            let s = if trim_integer && v.fract() == 0.0 {
                format!("{v:.0}")
            } else {
                format!("{v:.n$}")
            };
            if fmt.grouping { group_thousands(&s) } else { s }
        }
    }
}

// 1234.5 -> "1.235e+03"
fn format_sci(v: f64, n: usize) -> String {
    let s = format!("{v:.n$e}");
    match s.split_once('e') {
        Some((mantissa, exp)) => {
            let exp: i32 = exp.parse().unwrap_or(0);
            format!("{mantissa}e{exp:+03}")
        }
        None => s,
    }
}

// 12345 -> "12.345e+03" (指数を3の倍数にする)
fn format_eng(v: f64, n: usize) -> String {
    if v == 0.0 {
        return format!("{:.n$}e+00", 0.0);
    }
    let mut exp = (v.abs().log10().floor() as i32).div_euclid(3) * 3;
    let mut mantissa = format!("{:.n$}", v / 10f64.powi(exp));
    // 丸めで 1000.0 になった場合は次の桁へ繰り上げる
    if mantissa.trim_start_matches('-').parse::<f64>().is_ok_and(|m| m >= 1000.0) {
        exp += 3;
        mantissa = format!("{:.n$}", v / 10f64.powi(exp));
    }
    format!("{mantissa}e{exp:+03}")
}

// "-1234567.89" -> "-1,234,567.89"
fn group_thousands(s: &str) -> String {
    let (sign, rest) = match s.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", s),
    };
    let (int_part, frac_part) = match rest.find('.') {
        Some(i) => rest.split_at(i),
        None => (rest, ""),
    };
    let mut grouped = String::new();
    for (i, c) in int_part.chars().enumerate() {
        if i > 0 && (int_part.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(c);
    }
    format!("{sign}{grouped}{frac_part}")
}

impl CalcNum {
    pub fn num_format(&self, fmt: &DisplayFormat) -> String {
        match self {
            CalcNum::Number(val) => format_real(*val, fmt, true),
            CalcNum::Complex(val) => {
                // 3.000 + 4.000i / 3.000 - 4.000i (各部は小数部を省かず揃える)
                let sign = if val.im.is_sign_negative() { '-' } else { '+' };
                format!(
                    "{} {} {}i",
                    format_real(val.re, fmt, false),
                    sign,
                    format_real(val.im.abs(), fmt, false)
                )
            }
        }
    }

    fn is_integer(&self) -> bool {
        // 整数チェック
        match self {
            CalcNum::Number(val) => val.fract() == 0.0,
            _ => false,
        }
    }

    fn get_realnumber(&self) -> CalcResult<f64> {
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

    fn to_polar(&self, degmode: &DegMode) -> CalcResult<CalcNum> {
        // 極座標変換
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
    fn to_rectangular(&self, degmode: &DegMode) -> CalcResult<CalcNum> {
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

    fn to_deg(&self) -> CalcResult<CalcNum> {
        match self {
            CalcNum::Number(val) => Ok(CalcNum::Number(val.to_degrees())),
            CalcNum::Complex(_val) => Err("Complex number is not supported".into()),
        }
    }

    fn to_rad(&self) -> CalcResult<CalcNum> {
        match self {
            CalcNum::Number(val) => Ok(CalcNum::Number(val.to_radians())),
            CalcNum::Complex(_val) => Err("Complex number is not supported".into()),
        }
    }

    fn factorial(&self) -> CalcResult<CalcNum> {
        // 階乗計算 (f64 で表現できる 170! まで)
        if !self.is_integer() {
            return Err("Factorial is only supported for integer".into());
        }
        let n = self.get_realnumber()?;
        if n < 0.0 {
            return Err("Factorial is only supported for n >= 0".into());
        }
        if n > 170.0 {
            return Err("over flow".into());
        }
        Ok(CalcNum::Number(product_range(1, n as u64)?))
    }

    // 順列・組み合わせの引数チェック。整数かつ 0 <= r <= n であること
    fn n_r_args(&self, r: &Self, name: &str) -> CalcResult<(u64, u64)> {
        if !self.is_integer() || !r.is_integer() {
            return Err(format!("{name} is only supported for integer").into());
        }
        let (n, r) = (self.get_realnumber()?, r.get_realnumber()?);
        if n < 0.0 || r < 0.0 || r > n {
            return Err(format!("{name} requires 0 <= r <= n").into());
        }
        // f64 で整数を正確に表せる上限 (2^53) を超える n は扱わない
        if n > 9_007_199_254_740_992.0 {
            return Err(format!("{name}: n is too large").into());
        }
        Ok((n as u64, r as u64))
    }

    fn permutation(&self, r: &Self) -> CalcResult<CalcNum> {
        // 順列計算 nPr = n * (n-1) * ... * (n-r+1)
        let (n, r) = self.n_r_args(r, "Permutation")?;
        if r == 0 {
            return Ok(CalcNum::Number(1.0));
        }
        Ok(CalcNum::Number(product_range(n - r + 1, n)?))
    }

    fn combination(&self, r: &Self) -> CalcResult<CalcNum> {
        // 組み合わせ計算
        // nCr = Π_{k=1..r} (n-r+k)/k を逐次計算する。途中の値は常に C(n-r+k, k) で
        // 整数になるため、階乗同士の割り算と違い中間値が膨らまずに済む
        let (n, r) = self.n_r_args(r, "Combination")?;
        let r = r.min(n - r);
        let mut acc = 1.0_f64;
        for k in 1..=r {
            acc = acc * (n - r + k) as f64 / k as f64;
            if acc.is_infinite() {
                return Err("over flow".into());
            }
        }
        Ok(CalcNum::Number(acc.round()))
    }

    fn abs(&self) -> CalcNum {
        match self {
            CalcNum::Number(val) => CalcNum::Number(val.abs()),
            CalcNum::Complex(val) => CalcNum::Number(val.norm()),
        }
    }

    fn negate(&self) -> CalcNum {
        match self {
            CalcNum::Number(val) => CalcNum::Number(-val),
            CalcNum::Complex(val) => CalcNum::Complex(-val),
        }
    }

    fn exp(&self) -> CalcNum {
        match self {
            CalcNum::Number(val) => CalcNum::Number(val.exp()),
            CalcNum::Complex(val) => CalcNum::Complex(val.exp()),
        }
    }

    fn round(&self) -> CalcResult<CalcNum> {
        Ok(CalcNum::Number(self.get_realnumber()?.round()))
    }
}

// 計算の状態 (Undo のスナップショット単位)
#[derive(Debug, Clone, PartialEq)]
pub struct CalcState {
    pub stack: VecDeque<CalcNum>,
    pub memo_map: BTreeMap<String, CalcNum>,
    pub degmode: DegMode,
    // 直前の演算で消費した X (lastx で呼び戻す)
    pub last_x: Option<CalcNum>,
}

impl Default for CalcState {
    fn default() -> Self {
        Self {
            stack: VecDeque::new(),
            memo_map: BTreeMap::new(),
            degmode: DegMode::Deg,
            last_x: None,
        }
    }
}

pub const STACK_SIZE: usize = 12;

// スタックの管理関数
// 途中でエラーになっても状態が壊れないよう、作業用コピーで計算して成功時のみ反映する
pub fn manage_stack(
    expression: &str,
    state: &mut CalcState,
    memo_mode: &mut Option<Memorize>,
) -> CalcResult<()> {
    let mut work_state = state.clone();
    let mut work_memo_mode = memo_mode.clone();

    apply_expression(expression, &mut work_state, &mut work_memo_mode)?;

    // スタックが上限を超えた場合、古い要素(先頭)から削除
    while work_state.stack.len() > STACK_SIZE {
        work_state.stack.pop_front();
    }

    *state = work_state;
    *memo_mode = work_memo_mode;
    Ok(())
}

// 式を評価して状態を更新する
// エラー時に途中状態が残ってもよい (manage_stack が作業用コピーを破棄するため)
fn apply_expression(
    expression: &str,
    state: &mut CalcState,
    memo_mode: &mut Option<Memorize>,
) -> CalcResult<()> {
    // 空入力で最後尾のアイテムをコピー HP電卓の ENTER と同様
    if expression.trim().is_empty() {
        if let Some(top) = state.stack.back().cloned() {
            state.stack.push_back(top);
        }
        return Ok(());
    }

    // 先に全トークンを解釈してから順に実行する
    let items: Vec<Expr> = expression
        .split_whitespace()
        .flat_map(split_trailing_operator)
        .map(|token| parse_exp(&token, memo_mode))
        .collect();

    for item in items {
        match item {
            Expr::Numbers(num) => state.stack.push_back(num),
            Expr::Const(c) => state.stack.push_back(c.value()),
            Expr::Binomial(f) => apply_binomial(f, state)?,
            Expr::Monomial(f) => apply_monomial(f, state)?,
            Expr::Opstack(op) => apply_operate_stack(op, state)?,
            Expr::Memo(memo) => apply_memorize(memo, &mut state.stack, &mut state.memo_map)?,
        }
    }
    Ok(())
}

// "3 2/" のように末尾に付いた演算子を分離する ("2/" -> "2", "/")
fn split_trailing_operator(token: &str) -> Vec<String> {
    match token.chars().last() {
        Some('+' | '-' | '*' | '/' | '^') if token.len() > 1 => {
            let (head, tail) = token.split_at(token.len() - 1);
            vec![head.to_string(), tail.to_string()]
        }
        _ => vec![token.to_string()],
    }
}

// トークンを演算要素に変換する
// 優先順位: 数値 > コマンド表 > キー待ち中ならメモのキー > メモの呼び出し
fn parse_exp(token: &str, memo_mode: &mut Option<Memorize>) -> Expr {
    let lower = token.to_lowercase();
    if let Ok(num) = lower.parse::<CalcNum>() {
        return Expr::Numbers(num);
    }

    if let Some(command) = commands::find(&lower) {
        let expr = command.action.to_expr();
        // 'sto' 'rcl' などはキー待ちに入る
        if let Expr::Memo(memo) = &expr
            && memo.is_waiting_key()
        {
            *memo_mode = Some(memo.clone());
        }
        return expr;
    }

    // コマンドでなければメモのキーとして扱う (キーは大文字小文字を区別する)
    let key = token.to_string();
    Expr::Memo(match memo_mode.take() {
        Some(Memorize::Store(None)) => Memorize::Store(Some(key)),
        Some(Memorize::Delete(None)) => Memorize::Delete(Some(key)),
        Some(Memorize::Tvm(None)) => Memorize::Tvm(Some(key)),
        // Recall 待ち、またはキー待ちでなければ呼び出し
        _ => Memorize::Recall(Some(key)),
    })
}

// 二項演算: Y(2番目) と X(最上段) を取り出して結果を積む
fn apply_binomial(func: BinomialFunc, state: &mut CalcState) -> CalcResult<()> {
    let (y, x) = get_two_item(&mut state.stack)?;
    state.last_x = Some(x.clone());
    let result = match func {
        BinomialFunc::Add => y + x,
        BinomialFunc::Subtract => y - x,
        BinomialFunc::Multiply => y * x,
        BinomialFunc::Divide => y / x,
        BinomialFunc::Mod => y % x,
        BinomialFunc::Pow => y.pow(&x),
        BinomialFunc::NPr => y.permutation(&x)?,
        BinomialFunc::NCr => y.combination(&x)?,
    };
    state.stack.push_back(result);
    Ok(())
}

// 単項演算: X を取り出して結果を積む
fn apply_monomial(func: MonomialFunc, state: &mut CalcState) -> CalcResult<()> {
    let degmode = state.degmode;
    let x = get_one_item(&mut state.stack)?;
    state.last_x = Some(x.clone());
    let result = match func {
        MonomialFunc::Sqrt => x.sqrt(),
        MonomialFunc::Log => x.log10(),
        MonomialFunc::Ln => x.ln(),
        MonomialFunc::Sin => x.sin(&degmode),
        MonomialFunc::Cos => x.cos(&degmode),
        MonomialFunc::Tan => x.tan(&degmode),
        MonomialFunc::ASin => x.asin(&degmode),
        MonomialFunc::ACos => x.acos(&degmode),
        MonomialFunc::ATan => x.atan(&degmode),
        MonomialFunc::ToDeg => x.to_deg()?,
        MonomialFunc::ToRad => x.to_rad()?,
        MonomialFunc::Abs => x.abs(),
        MonomialFunc::Factorial => x.factorial()?,
        MonomialFunc::ToPolar => x.to_polar(&degmode)?,
        MonomialFunc::ToRec => x.to_rectangular(&degmode)?,
        MonomialFunc::Chs => x.negate(),
        MonomialFunc::Inv => CalcNum::Number(1.0) / x,
        MonomialFunc::Square => x.clone() * x,
        MonomialFunc::Exp => x.exp(),
        MonomialFunc::Pow10 => CalcNum::Number(10.0).pow(&x),
        MonomialFunc::Round => x.round()?,
    };
    state.stack.push_back(result);
    Ok(())
}

// スタック操作・モード切替
fn apply_operate_stack(op: OperateStack, state: &mut CalcState) -> CalcResult<()> {
    let calstack = &mut state.stack;
    match op {
        OperateStack::Swap => {
            let len = require_len(calstack, 2)?;
            calstack.swap(len - 1, len - 2);
        }
        OperateStack::Clear => calstack.clear(),
        OperateStack::Delete => {
            get_one_item(calstack)?;
        }
        OperateStack::RollUp => {
            // 最上段を最下段へ
            require_len(calstack, 2)?;
            calstack.rotate_right(1);
        }
        OperateStack::RollDown => {
            // 最下段を最上段へ
            require_len(calstack, 2)?;
            calstack.rotate_left(1);
        }
        OperateStack::Sum => {
            let total = calstack
                .iter()
                .map(CalcNum::get_realnumber)
                .sum::<CalcResult<f64>>()?;
            calstack.clear();
            calstack.push_back(CalcNum::Number(total));
        }
        OperateStack::LastX => {
            let last_x = state.last_x.clone().ok_or("No last X")?;
            calstack.push_back(last_x);
        }
        OperateStack::Deg => state.degmode = DegMode::Deg,
        OperateStack::Rad => state.degmode = DegMode::Rad,
    }
    Ok(())
}

// メモリ操作
fn apply_memorize(
    memo: Memorize,
    calstack: &mut VecDeque<CalcNum>,
    memory_map: &mut BTreeMap<String, CalcNum>,
) -> CalcResult<()> {
    match memo {
        // キー待ちに入るだけ (キーは次のトークンで受け取る)
        Memorize::Recall(None)
        | Memorize::Store(None)
        | Memorize::Delete(None)
        | Memorize::Tvm(None) => {}
        Memorize::Recall(Some(key)) => {
            let val = memory_map
                .get(&key)
                .ok_or_else(|| format!("Unknown command or memo key: '{key}'"))?;
            calstack.push_back(val.clone());
        }
        Memorize::Store(Some(key)) => {
            let top = calstack.back().ok_or("Stack is Empty")?.clone();
            memory_map.insert(key, top);
        }
        Memorize::Delete(Some(key)) => {
            memory_map
                .remove(&key)
                .ok_or_else(|| format!("No memo key: '{key}'"))?;
        }
        Memorize::Clear => memory_map.clear(),
        Memorize::Tvm(Some(key)) => {
            let value = CalcNum::Number(solve_tvm(memory_map, &key)?);
            calstack.push_back(value.clone());
            memory_map.insert(key, value);
        }
    }
    Ok(())
}

// TVM: target 以外の4つの値をメモリから取り出して target を求める
fn solve_tvm(memory_map: &BTreeMap<String, CalcNum>, target: &str) -> Result<f64, String> {
    let get = |key: &str| {
        memory_map
            .get(key)
            .and_then(|v| v.get_realnumber().ok())
            .ok_or(format!("tvm: '{key}' is not set"))
    };
    let value = match target {
        "n" => finance::number_of_periods(get("iyr")?, get("pv")?, get("pmt")?, get("fv")?),
        "iyr" => finance::i_yr(get("n")?, get("pv")?, get("pmt")?, get("fv")?)
            .ok_or("tvm: no solution for iyr")?,
        "pv" => finance::present_value(get("n")?, get("iyr")?, get("pmt")?, get("fv")?),
        "pmt" => finance::payment(get("n")?, get("iyr")?, get("pv")?, get("fv")?),
        "fv" => finance::future_value(get("n")?, get("iyr")?, get("pv")?, get("pmt")?),
        _ => return Err(format!("tvm: unknown target '{target}' (n, iyr, pv, pmt, fv)")),
    };
    // 0除算や対数の定義域外で NaN / inf になった場合は解なし
    if !value.is_finite() {
        return Err(format!("tvm: no solution for {target}"));
    }
    Ok(value)
}

// スタックに n 個以上あることを確認し、長さを返す
fn require_len(calstack: &VecDeque<CalcNum>, n: usize) -> Result<usize, String> {
    if calstack.len() < n {
        Err("Stack is too short".to_string())
    } else {
        Ok(calstack.len())
    }
}

// スタックから2つの要素を (Y, X) の順で取り出す
fn get_two_item(calstack: &mut VecDeque<CalcNum>) -> Result<(CalcNum, CalcNum), String> {
    require_len(calstack, 2)?;
    let x = calstack.pop_back().unwrap();
    let y = calstack.pop_back().unwrap();
    Ok((y, x))
}

// スタックから1つの要素を取り出す
fn get_one_item(calstack: &mut VecDeque<CalcNum>) -> Result<CalcNum, String> {
    calstack.pop_back().ok_or_else(|| "Stack is Empty".to_string())
}

fn safe_product(iter: impl IntoIterator<Item = u64>) -> Result<u64, &'static str> {
    iter.into_iter()
        .try_fold(1u64, |acc, x| acc.checked_mul(x).ok_or("over flow"))
}

// start..=end の積 (start >= 1 を前提とする)
// u64 に収まる範囲は整数で正確に計算し、溢れたら f64 で近似計算する
fn product_range(start: u64, end: u64) -> Result<f64, &'static str> {
    if let Ok(v) = safe_product(start..=end) {
        return Ok(v as f64);
    }
    let mut acc = 1.0_f64;
    for x in start..=end {
        acc *= x as f64;
        if acc.is_infinite() {
            return Err("over flow");
        }
    }
    Ok(acc)
}

#[cfg(test)]
mod tests {

    use super::{CalcNum, CalcState, DegMode, DisplayFormat, Notation, STACK_SIZE, manage_stack};
    use std::collections::{BTreeMap, VecDeque};
    use std::f64::consts;

    #[test]
    fn test() -> Result<(), String> {
        let test_manage = |exp| {
            let mut state = rad_state();
            let mut memomode = None;
            match manage_stack(exp, &mut state, &mut memomode) {
                Ok(_) => (),
                Err(e) => eprintln!("{e}"),
            };
            match &state.stack[0] {
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
        assert_eq!(realnumtest("10.0 ln"), consts::LN_10);
        assert_eq!(realnumtest("100.0 log"), 2.0);
        assert_eq!(realnumtest("9.0 3 3 5 sum"), 20.0);
        assert!((realnumtest("pi 6 / sin") - 0.5).abs() < 1e-10);
        assert!((realnumtest("pi 3 / cos") - 0.5).abs() < 1e-10);
        assert!((realnumtest("pi 4 / tan") - 1.0).abs() < 1e-10);
        assert!((realnumtest("0.5 asin") - consts::PI / 6.0).abs() < 1e-10);
        assert!((realnumtest("0.5 acos") - consts::PI / 3.0).abs() < 1e-10);
        assert!((realnumtest("1.0 atan") - consts::PI / 4.0).abs() < 1e-10);
        assert_eq!(realnumtest("pi"), consts::PI);
        assert_eq!(realnumtest("e"), consts::E);
        assert!((realnumtest("60 torad") - consts::PI / 3.0).abs() < 1e-10);
        assert!((realnumtest("pi 3 / todeg") - 60.0).abs() < 1e-10);
        assert_eq!(realnumtest("2 3 + 12 *"), 60.0);
        assert_eq!(realnumtest("3 3 3 sum sqrt"), 3.0);
        assert_eq!(realnumtest("10 3 npr"), 720.0);
        assert_eq!(realnumtest("10 3 ncr"), 120.0);
        assert_eq!(realnumtest("5 n!"), 120.0);
        complex_assert(complexnumtest("-10i log"), (1.0, -0.6821881769));
        complex_assert(
            complexnumtest("-10i ln"),
            (std::f64::consts::LN_10, -std::f64::consts::FRAC_PI_2),
        );
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

    // 空の状態から式を評価し、(結果, スタック, メモリ) を返す
    fn run(exp: &str) -> (Result<(), String>, VecDeque<CalcNum>, BTreeMap<String, CalcNum>) {
        let mut state = rad_state();
        let mut memomode = None;
        let result = manage_stack(exp, &mut state, &mut memomode).map_err(|e| e.to_string());
        (result, state.stack, state.memo_map)
    }

    // テストはラジアンモードで行う
    fn rad_state() -> CalcState {
        CalcState {
            degmode: DegMode::Rad,
            ..CalcState::default()
        }
    }

    fn top(stack: &VecDeque<CalcNum>) -> f64 {
        match stack.back() {
            Some(CalcNum::Number(v)) => *v,
            other => panic!("expected real number on top, got {other:?}"),
        }
    }

    fn fmt(notation: Notation, decimals: usize, grouping: bool) -> DisplayFormat {
        DisplayFormat {
            notation,
            decimals,
            grouping,
        }
    }

    fn show(v: f64, f: DisplayFormat) -> String {
        CalcNum::Number(v).num_format(&f)
    }

    #[test]
    fn complex_format() {
        use num::complex::Complex;
        let f = fmt(Notation::Fix, 2, true);
        assert_eq!(CalcNum::Complex(Complex::new(3.0, -4.0)).num_format(&f), "3.00 - 4.00i");
        let f = fmt(Notation::Fix, 1, true);
        assert_eq!(CalcNum::Complex(Complex::new(1.5, 2.0)).num_format(&f), "1.5 + 2.0i");
    }

    #[test]
    fn fix_format() {
        let f = fmt(Notation::Fix, 3, true);
        assert_eq!(show(3.0, f), "3");
        assert_eq!(show(0.5, f), "0.500");
        assert_eq!(show(1234567.0, f), "1,234,567");
        assert_eq!(show(-1234.5, f), "-1,234.500");
        assert_eq!(show(123.0, f), "123");
        assert_eq!(show(-0.0, f), "0");
        // 極端な値は指数表記
        assert_eq!(show(1e300, f), "1.000e+300");
        assert_eq!(show(1e-5, f), "1.000e-05");
        assert_eq!(show(0.0004, f), "4.000e-04");
        assert_eq!(show(f64::INFINITY, f), "inf");
        assert_eq!(show(f64::NAN, f), "NaN");
        // 区切りなし
        assert_eq!(show(1234567.25, fmt(Notation::Fix, 2, false)), "1234567.25");
    }

    #[test]
    fn sci_and_eng_format() {
        let sci = fmt(Notation::Sci, 2, true);
        // (丸めの境界 x.xx5 は2進数の表現誤差で結果が揺れるため避ける)
        assert_eq!(show(12346.0, sci), "1.23e+04");
        assert_eq!(show(-0.00012, sci), "-1.20e-04");
        assert_eq!(show(0.0, sci), "0.00e+00");

        let eng = fmt(Notation::Eng, 2, true);
        assert_eq!(show(12346.0, eng), "12.35e+03");
        assert_eq!(show(0.01, eng), "10.00e-03");
        assert_eq!(show(1.5, eng), "1.50e+00");
        assert_eq!(show(-470000.0, eng), "-470.00e+03");
        // 丸めで 1000 になる場合は繰り上げ
        assert_eq!(show(999999.0, eng), "1.00e+06");
        assert_eq!(show(0.0, eng), "0.00e+00");
    }

    #[test]
    fn error_keeps_state_unchanged() {
        // 途中でエラーになった場合、スタック・メモリ・モードは入力前のまま
        let mut state = rad_state();
        state.stack.push_back(CalcNum::Number(10.0));
        let before = state.clone();
        let mut memomode = None;

        let result = manage_stack("1 2 + sto x rad deg unknown_key", &mut state, &mut memomode);
        assert!(result.is_err());
        assert_eq!(state, before);
    }

    #[test]
    fn new_functions() {
        let (_, stack, _) = run("5 chs");
        assert_eq!(top(&stack), -5.0);
        let (_, stack, _) = run("4 inv");
        assert_eq!(top(&stack), 0.25);
        let (_, stack, _) = run("3 sq");
        assert_eq!(top(&stack), 9.0);
        let (_, stack, _) = run("0 exp");
        assert_eq!(top(&stack), 1.0);
        let (_, stack, _) = run("3 10^x");
        assert!((top(&stack) - 1000.0).abs() < 1e-9);
        let (_, stack, _) = run("2.5 round -2.5 round");
        assert_eq!(stack, VecDeque::from([3.0, -3.0].map(CalcNum::Number)));
        assert!(run("1+2i round").0.is_err());
        // 複素数にも対応
        let (_, stack, _) = run("3+4i chs");
        assert_eq!(stack.back(), Some(&CalcNum::Complex(num::complex::Complex::new(-3.0, -4.0))));
    }

    #[test]
    fn last_x() {
        // 二項演算・単項演算で消費した X を呼び戻せる
        let (_, stack, _) = run("10 4 - lastx");
        assert_eq!(stack, VecDeque::from([6.0, 4.0].map(CalcNum::Number)));
        let (_, stack, _) = run("9 sqrt lastx");
        assert_eq!(stack, VecDeque::from([3.0, 9.0].map(CalcNum::Number)));
        // まだ演算していなければエラー
        assert!(run("1 lastx").0.is_err());
    }

    #[test]
    fn factorial_permutation_combination() {
        let num = |v: f64| CalcNum::Number(v);
        let real = |r: Result<CalcNum, Box<dyn std::error::Error>>| match r.unwrap() {
            CalcNum::Number(v) => v,
            CalcNum::Complex(_) => panic!("unexpected complex"),
        };

        // 境界値
        assert_eq!(real(num(0.0).factorial()), 1.0);
        assert_eq!(real(num(5.0).permutation(&num(0.0))), 1.0);
        assert_eq!(real(num(5.0).combination(&num(0.0))), 1.0);
        assert_eq!(real(num(5.0).combination(&num(5.0))), 1.0);
        assert_eq!(real(num(10.0).combination(&num(7.0))), 120.0);

        // u64 では溢れるが f64 で表現できる値 (以前は over flow エラー)
        assert_eq!(real(num(52.0).combination(&num(5.0))), 2_598_960.0);
        let c60_30 = real(num(60.0).combination(&num(30.0)));
        assert!((c60_30 / 1.182_645_815_648_614_2e17 - 1.0).abs() < 1e-12);
        let p25_25 = real(num(25.0).permutation(&num(25.0)));
        let f25 = real(num(25.0).factorial());
        assert!((p25_25 / f25 - 1.0).abs() < 1e-12);
        assert!(num(170.0).factorial().is_ok());

        // エラーになるべきケース
        assert!(num(171.0).factorial().is_err());
        assert!(num(-1.0).factorial().is_err());
        assert!(num(2.5).factorial().is_err());
        assert!(num(3.0).combination(&num(5.0)).is_err());
        assert!(num(5.0).permutation(&num(-1.0)).is_err());
        assert!(num(2000.0).combination(&num(1000.0)).is_err());
    }

    #[test]
    fn stack_is_limited_to_stack_size() {
        let input = (1..=20).map(|n| n.to_string()).collect::<Vec<_>>().join(" ");
        let (result, stack, _) = run(&input);
        assert!(result.is_ok());
        assert_eq!(stack.len(), STACK_SIZE);
        // 古いものから削除され、最新の 20 が末尾に残る
        assert_eq!(top(&stack), 20.0);
    }

    #[test]
    fn stack_operations() {
        let (_, stack, _) = run("1 2 3 sw");
        assert_eq!(stack, VecDeque::from([1.0, 3.0, 2.0].map(CalcNum::Number)));
        let (_, stack, _) = run("1 2 3 rup");
        assert_eq!(stack, VecDeque::from([3.0, 1.0, 2.0].map(CalcNum::Number)));
        let (_, stack, _) = run("1 2 3 rdn");
        assert_eq!(stack, VecDeque::from([2.0, 3.0, 1.0].map(CalcNum::Number)));
        let (_, stack, _) = run("1 2 3 dl");
        assert_eq!(stack, VecDeque::from([1.0, 2.0].map(CalcNum::Number)));
        let (_, stack, _) = run("5 ");
        assert_eq!(top(&stack), 5.0);
        assert!(run("1 sw").0.is_err());
        assert!(run("dl").0.is_err());
        assert!(run("1 2+3i sum").0.is_err());
    }

    #[test]
    fn commands_are_case_insensitive() {
        let (_, stack, _) = run("PI 2 / SIN");
        assert!((top(&stack) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn unknown_token_reports_clearly() {
        let (result, _, _) = run("1 foo");
        assert!(result.unwrap_err().contains("Unknown command or memo key: 'foo'"));
    }

    #[test]
    fn memory_store_recall_delete() {
        // 明示的な rcl と、キー名だけでの呼び出しの両方
        let (_, stack, memory) = run("5 sto x rcl x x +");
        assert_eq!(top(&stack), 10.0);
        assert_eq!(memory.get("x"), Some(&CalcNum::Number(5.0)));

        let (_, _, memory) = run("5 sto x 6 sto y mdel x");
        assert!(!memory.contains_key("x"));
        assert!(memory.contains_key("y"));

        let (_, _, memory) = run("5 sto x mc");
        assert!(memory.is_empty());

        assert!(run("mdel nothing").0.is_err());
        assert!(run("sto x").0.is_err()); // スタックが空
    }

    #[test]
    fn tvm_solve() {
        let (result, stack, memory) = run("12 tn 20 tiyr -22 tpv -1 tpmt tvm fv");
        assert!(result.is_ok());
        assert!((top(&stack) - 39.99).abs() < 0.01);
        assert!(memory.contains_key("fv"));

        let (result, _, _) = run("12 tn 20 tiyr tvm fv");
        assert!(result.unwrap_err().contains("'pv' is not set"));

        let (result, _, _) = run("12 tn tvm xyz");
        assert!(result.unwrap_err().contains("unknown target"));
    }
}
