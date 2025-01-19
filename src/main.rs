mod calcenums;
use calcenums::{manage_stack, DegMode, Memorize};

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
};
#[derive(Debug, Clone, Copy)]
enum StackData {
    Number(f64),
    Complex(f64, f64),
}
// impl std::fmt::Display for StackData {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             StackData::Number(val) => write!(f, "{:.4}", val),
//             StackData::Complex(re, im) => write!(f, "{:.4} + {:.4}i", re, im),
//         }
//     }
// }

impl StackData {
    fn float_format(&self, n_place: usize) -> String {
        match self {
            StackData::Number(val) => format!("{:.1$}", val, n_place),
            StackData::Complex(re, im) => format!("{:.2$} + {:.2$}i", re, im, n_place),
        }
    }
    fn is_number(&self) -> bool {
        matches!(self, StackData::Number(_))
    }
    fn is_complex(&self) -> bool {
        matches!(self, StackData::Complex(_, _))
    }
    fn get_number(&self) -> f64 {
        match self {
            StackData::Number(val) => *val,
            _ => 0.0,
        }
    }
}

const RESULT_DISPLAY_LENGTH: u16 = 12;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, DisableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut stack: VecDeque<StackData> = VecDeque::new();
    let mut memo_map: BTreeMap<String, StackData> = BTreeMap::new();
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
        let result_len = RESULT_DISPLAY_LENGTH.min(stack.len() as u16) + 2;
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(1),
                        Constraint::Length(4),
                        Constraint::Length(result_len),
                        Constraint::Length(2),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            //Memory
            let memory_block = Block::default().borders(Borders::NONE);
            let memory_text = Paragraph::new(memory.as_ref()).block(memory_block);
            f.render_widget(memory_text, chunks[1]);

            // ステータスバー
            let status_block = Block::default().borders(Borders::NONE);
            let status_text = Paragraph::new(format!(
                "Fix:{} | {:?} | MemoMode:{:?}",
                decimal_point, degmode, memo_mode
            ))
            .block(status_block);
            f.render_widget(status_text, chunks[0]);
            //結果表示
            let result_block = Block::default().title("Result Stack").borders(Borders::ALL);
            let result_text = Paragraph::new(result.as_ref()).block(result_block);
            f.render_widget(result_text, chunks[3]);

            // 入力
            let input_block = Block::default().title("Input ").borders(Borders::TOP);
            let input_text = Paragraph::new(input.as_ref()).block(input_block);
            f.render_widget(input_text, chunks[4]);
            // Helper メッセージ
            let help_block = Block::default().title("Message").borders(Borders::ALL);
            let help_text = Paragraph::new(format!(
                "Enter: Calc | Esc : Quit | Undo : undo{}{}",
                sepalator, message
            ))
            .block(help_block);
            f.render_widget(help_text, chunks[2]);
        })?;

        // 入力のカーソルを表示
        let input_col = 1;
        let input_row = result_len + 8;
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
        .fold("Hist: ".to_string(), |acc, x| acc + x + " =>");
}

fn update_memo(memo_map: &BTreeMap<String, StackData>, memory: &mut String) {
    // memo_mapをStringに変換しスペースで区切る
    for (key, sval) in memo_map.iter() {
        memory.push_str(&format!("{}: {} ", key, sval.float_format(2)));
    }
}

fn update_stack(stack: &VecDeque<StackData>, result: &mut String, decimal_point: usize) {
    // 改行で区切る。windowsの場合は\r\n, Mac|linuxの場合は\n
    let sepalator = if cfg!(target_os = "windows") {
        "\r\n"
    } else {
        "\n"
    };
    for (i, sval) in stack.iter().enumerate().rev().take(10).rev() {
        result.push_str(sval.float_format(decimal_point).as_ref());
        if i < stack.len() - 1 {
            result.push_str(sepalator);
        }
    }
}
