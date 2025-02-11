mod calcrpn;
use calcrpn::{manage_stack, DegMode, Memorize};
use num::complex::Complex;
use std::ops::{Add, Div, Mul, Rem, Sub};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use rustyline::{
    config::Configurer,
    history::{History, SearchDirection},
    DefaultEditor,
};
use std::{
    collections::{BTreeMap, VecDeque},
    io,
    str::FromStr,
};

#[derive(Debug, Clone)]
enum CalcNum {
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
    fn num_format(&self, n_place: usize) -> String {
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

    fn get_realnumber(&self) -> Result<f64, String> {
        match self {
            CalcNum::Number(val) => Ok(*val),
            CalcNum::Complex(_val) => Err("Complex number is not supported".to_string()),
        }
    }
    fn pow(&self, n: &Self) -> Result<CalcNum, String> {
        // if !n.is_realnumber() {
        //     return Err("Exponent must be real number".to_string());
        // };
        // let n = n.get_realnumber()?;
        match (self, n) {
            (CalcNum::Number(val), CalcNum::Number(n)) => Ok(CalcNum::Number(val.powf(*n))),
            (CalcNum::Complex(val), CalcNum::Number(n)) => Ok(CalcNum::Complex(val.powf(*n))),
            (CalcNum::Number(val), CalcNum::Complex(n)) => {
                Ok(CalcNum::Complex(Complex::new(*val, 0.0).powc(*n)))
            }
            (CalcNum::Complex(val), CalcNum::Complex(n)) => Ok(CalcNum::Complex(val.powc(*n))),
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
    fn to_deg(&self) -> Result<CalcNum, String> {
        match self {
            CalcNum::Number(val) => Ok(CalcNum::Number(val.to_degrees())),
            CalcNum::Complex(_val) => Err("Complex number is not supported".to_string()),
        }
    }

    fn to_rad(&self) -> Result<CalcNum, String> {
        match self {
            CalcNum::Number(val) => Ok(CalcNum::Number(val.to_radians())),
            CalcNum::Complex(_val) => Err("Complex number is not supported".to_string()),
        }
    }

    fn factorial(&self) -> Result<CalcNum, String> {
        // 階乗計算
        if !self.is_integer() {
            return Err("Factorial is only supported for integer".to_string());
        }
        match self {
            CalcNum::Number(val) => Ok(CalcNum::Number((1..=*val as u64).product::<u64>() as f64)),
            CalcNum::Complex(_val) => Err("Complex number is not supported".to_string()),
        }
    }

    fn permutation(&self, n: &Self) -> Result<CalcNum, String> {
        // 順列計算
        if !self.is_integer() || !n.is_integer() {
            return Err("Permutation is only supported for integer".to_string());
        }
        match self {
            CalcNum::Number(val) => Ok(CalcNum::Number(
                (1..=*val as u64)
                    .rev()
                    .take(n.get_realnumber()? as usize)
                    .product::<u64>() as f64,
            )),
            CalcNum::Complex(_val) => Err("Complex number is not supported".to_string()),
        }
    }

    fn combination(&self, n: &Self) -> Result<CalcNum, String> {
        // 組み合わせ計算
        if !self.is_integer() || !n.is_integer() {
            return Err("Combination is only supported for integer".to_string());
        }
        match self {
            CalcNum::Number(val) => Ok(CalcNum::Number(
                (1..=*val as u64)
                    .rev()
                    .take(n.get_realnumber()? as usize)
                    .product::<u64>() as f64
                    / (1..=n.get_realnumber()? as u64).product::<u64>() as f64,
            )),
            CalcNum::Complex(_val) => Err("Complex number is not supported".to_string()),
        }
    }

    fn abs(&self) -> CalcNum {
        match self {
            CalcNum::Number(val) => CalcNum::Number(val.abs()),
            CalcNum::Complex(val) => CalcNum::Number(val.norm()),
        }
    }
}

// const RESULT_DISPLAY_LENGTH: u16 = 12;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, DisableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut stack: VecDeque<CalcNum> = VecDeque::new();
    let mut memo_map: BTreeMap<String, CalcNum> = BTreeMap::new();
    let mut degmode = DegMode::Deg;
    let mut memo_mode: Option<Memorize> = None;
    let mut input = String::new();
    let mut readline = DefaultEditor::new()?;
    readline.set_max_history_size(20)?;
    let mut input_log: VecDeque<String> = VecDeque::new();
    let mut result = String::new();
    let mut memory = String::new();
    let mut message = String::new();
    let mut decimal_point: usize = 3;
    let mut last_result = (stack.clone(), result.clone());
    let mut hist_index: usize = 0;
    let sepalator = if cfg!(target_os = "windows") {
        "\r\n"
    } else {
        "\n"
    };

    loop {
        let result_len = calcrpn::STACK_SIZE.min(stack.len()) + 2;
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(4),
                        Constraint::Length(result_len as u16),
                        Constraint::Length(2),
                    ]
                    .as_ref(),
                )
                .split(f.size());
            // ステータスバー
            let status_block = Block::default().borders(Borders::NONE);
            let status_text = Paragraph::new(format!(
                "Fix:{} | {:?} | MemoMode:{:?}",
                decimal_point, degmode, memo_mode
            ))
            .block(status_block);
            f.render_widget(status_text, chunks[0]);

            //Memory
            let memory_block = Block::default().borders(Borders::NONE);
            let memory_text = Paragraph::new(memory.as_ref()).block(memory_block);
            f.render_widget(memory_text, chunks[1]);
            // Helper メッセージ
            let help_block = Block::default().title("Message").borders(Borders::ALL);
            let help_text = Paragraph::new(format!(
                "Enter: Calc | Esc : Quit | Undo : undo{}{}",
                sepalator, message
            ))
            .block(help_block);
            f.render_widget(help_text, chunks[2]);

            //結果表示
            let result_block = Block::default().title("Result Stack").borders(Borders::ALL);
            let result_text = Paragraph::new(result.as_ref()).block(result_block);
            f.render_widget(result_text, chunks[3]);

            // 入力
            let input_block = Block::default().title("Input ").borders(Borders::TOP);
            let input_text = Paragraph::new(input.as_ref()).block(input_block);
            f.render_widget(input_text, chunks[4]);
        })?;

        // 入力のカーソルを表示
        let input_col = 1;
        let input_row = result_len as u16 + 8;
        let cursor_col = input_col + input.len() as u16;

        execute!(
            terminal.backend_mut(),
            crossterm::cursor::MoveTo(cursor_col, input_row),
            crossterm::cursor::Show
        )?;

        // 入力待ち
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char(c) => {
                    input.push(c);
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                // readline入力履歴機能
                KeyCode::Up => {
                    if !readline.history().is_empty() {
                        hist_index = hist_index.saturating_sub(1);

                        if let Some(prev) = readline
                            .history()
                            .get(hist_index, SearchDirection::Reverse)?
                        {
                            input = prev.entry.to_string();
                        };
                    }
                }
                KeyCode::Down => {
                    if !readline.history().is_empty() {
                        hist_index = (readline.history().len() - 1).min(hist_index + 1);
                        if let Some(prev) = readline
                            .history()
                            .get(hist_index, SearchDirection::Reverse)?
                        {
                            input = prev.entry.to_string();
                        };
                    }
                }
                // readline入力履歴機能ここまで
                KeyCode::Enter => {
                    hist_index = readline.history().len() + 1;
                    input_log.push_back(input.clone());
                    if &input == "undo" {
                        // undoの処理
                        stack = last_result.0.clone();
                        result = last_result.1.clone();
                        input.clear();
                        message = "Undo".to_string();
                        continue;
                    } else if &input == "clear" {
                        // clearの処理
                        stack.clear();
                        result.clear();
                        input.clear();
                        message = "Clear".to_string();
                        continue;
                    } else {
                        let app_command = input.split_whitespace().collect::<Vec<&str>>();
                        if app_command.len() == 2 {
                            if app_command[0] == "fix" {
                                // fixの処理
                                let fix = app_command[1].parse::<usize>().unwrap_or(3);
                                decimal_point = fix;
                                input.clear();
                            } else if app_command[0] == "clv" {
                                memo_map.remove_entry(app_command[1]);
                                update_log(&mut input_log, &mut message);
                                update_memo(&memo_map, &mut memory);
                                input.clear();
                            }
                        }

                        let temp_stack = stack.clone();
                        let temp_result = result.clone();

                        match manage_stack(
                            &input,
                            &mut stack,
                            &mut degmode,
                            &mut memo_map,
                            &mut memo_mode,
                        ) {
                            Ok(()) => {
                                result.clear();
                                memory.clear();
                                // 入力を履歴に追加
                                let line = input.clone();
                                readline.add_history_entry(line.as_str())?;
                                // スタックを更新
                                update_stack(&stack, &mut result, decimal_point);
                                update_memo(&memo_map, &mut memory);
                                // undo用スタックに保持
                                last_result = (temp_stack, temp_result);
                                update_log(&mut input_log, &mut message);
                                input.clear();
                            }
                            Err(e) => {
                                message = format!("Error: {}", e);
                                input_log.pop_back();
                                input.clear();
                            }
                        }
                    }
                }
                KeyCode::Esc => {
                    break;
                }
                _ => {}
            }
        }
    }

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    disable_raw_mode()?;
    terminal.show_cursor()?;
    Ok(())
}

fn update_log(input_log: &mut VecDeque<String>, message: &mut String) {
    if input_log.len() > 8 {
        input_log.pop_front();
    }
    *message = input_log
        .iter()
        .rev()
        .fold("Hist: ".to_string(), |acc, x| acc + x + " ← ");
}

fn update_memo(memo_map: &BTreeMap<String, CalcNum>, memory: &mut String) {
    // memo_mapをStringに変換しスペースで区切る
    for (key, sval) in memo_map.iter() {
        memory.push_str(&format!("{}: {} ", key, sval.num_format(2)));
    }
}

fn update_stack(stack: &VecDeque<CalcNum>, result: &mut String, decimal_point: usize) {
    // 改行で区切る。windowsの場合は\r\n, Mac|linuxの場合は\n
    let sepalator = if cfg!(target_os = "windows") {
        "\r\n"
    } else {
        "\n"
    };
    for (i, sval) in stack.iter().enumerate().rev().take(10).rev() {
        result.push_str(sval.num_format(decimal_point).as_ref());
        if i < stack.len() - 1 {
            result.push_str(sepalator);
        }
    }
}
