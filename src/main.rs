mod calcrpn;
mod finance;
use calcrpn::{CalcNum, DegMode, Memorize, manage_stack};
use crossterm::execute;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
};
use rustyline::{DefaultEditor, config::Configurer};
use std::{
    collections::{BTreeMap, VecDeque},
    io,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut terminal = ratatui::init();
    let result = run(&mut terminal);
    ratatui::restore();
    result
}

fn run(terminal: &mut ratatui::DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
    let mut stack: VecDeque<CalcNum> = VecDeque::new();
    let mut memo_map: BTreeMap<String, CalcNum> = BTreeMap::new();
    let mut degmode = DegMode::Deg;
    let mut memo_mode: Option<Memorize> = None;
    let mut result = String::new();
    let mut memory = String::new();
    let mut message = String::new();
    let mut decimal_point: usize = 3;
    let mut last_stackresult = vec![(stack.clone(), result.clone())];
    let mut do_continue = true;
    let mut input = String::new();
    let mut readline = DefaultEditor::new()?;
    let mut input_log: VecDeque<String> = VecDeque::new();

    readline.set_max_history_size(20)?;

    let sepalator = if cfg!(target_os = "windows") {
        "\r\n"
    } else {
        "\n"
    };

    while do_continue {
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
                            Constraint::Length(2),
                        ]
                        .as_ref(),
                    )
                    .split(f.area());
                // ステータスバー
                let status_block = Block::default().borders(Borders::NONE);
                let status_text = Paragraph::new(format!(
                    "Fix:{decimal_point} | {degmode:?} | MemoMode:{memo_mode:?}",
                ))
                .block(status_block);

                f.render_widget(status_text, chunks[0]);

                //Memory
                let memory_block = Block::default().borders(Borders::NONE);
                let memory_text = Paragraph::new(memory.clone()).block(memory_block);

                f.render_widget(memory_text, chunks[1]);
                // Helper メッセージ
                let help_block = Block::default().title("Message").borders(Borders::ALL);
                let help_text = Paragraph::new(format!(
                    "Enter: Calc | quit or q : Quit | Undo : undo{sepalator}{message}",
                ))
                .block(help_block);

                f.render_widget(help_text, chunks[2]);

                //結果表示
                let result_block = Block::default().title("Result Stack").borders(Borders::ALL);
                let result_text = Paragraph::new(result.clone()).block(result_block);
                f.render_widget(result_text, chunks[3]);

                // 入力
                let input_block = Block::default().title("Input").borders(Borders::TOP);
                let input_text = Paragraph::new(input.clone()).block(input_block);
                f.render_widget(input_text, chunks[4]);
                // 余白
                let status_block = Block::default().borders(Borders::TOP);
                let status_text = Paragraph::new(" ".to_string()).block(status_block);
                f.render_widget(status_text, chunks[5]);
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

            input = readline.readline(" >> ")?;
            if !input.is_empty() {
                input_log.push_back(input.clone());
            }

            match input.trim() {
                "undo" => {
                    // undoの処理
                    last_stackresult.pop();
                    if let Some((st, rst)) = last_stackresult.pop() {
                        stack = st;
                        result = rst;
                        update_stack(&stack, &mut result, decimal_point);
                    };
                    message = "Undo".to_string();
                    continue;
                }
                "help" => {
                    break;
                }
                "quit" | "q" => {
                    do_continue = false;
                    break;
                }
                "clh" | "clearhist" => {
                    input_log.clear();
                    terminal.clear()?;
                }
                _ => {
                    let app_command = input.split_whitespace().collect::<Vec<&str>>();
                    if app_command.len() == 2 {
                        // 'fix 2'のように引数を取るコマンド
                        match app_command[0] {
                            "fix" => {
                                // fixの処理
                                let fix = app_command[1].parse::<usize>().unwrap_or(3);
                                decimal_point = fix;
                            }
                            "clv" => {
                                // 特定のmemo keyを削除
                                memo_map.remove_entry(app_command[1]);
                                update_log(&mut input_log, &mut message);
                                update_memo(&memo_map, &mut memory);
                                terminal.clear()?;
                            }
                            _ => (),
                        }
                    }
                    let pre_stack_length = &stack.len();

                    match manage_stack(
                        &input,
                        &mut stack,
                        &mut degmode,
                        &mut memo_map,
                        &mut memo_mode,
                    ) {
                        Ok(()) => {
                            // 入力を履歴に追加
                            readline.add_history_entry(&input)?;
                            update_log(&mut input_log, &mut message);
                            last_stackresult.push((stack.clone(), result.clone()));
                            if last_stackresult.len() > 4 {
                                last_stackresult = last_stackresult[1..].to_vec();
                            }
                        }
                        Err(e) => {
                            message = format!("Error: {e}");
                            input_log.pop_back();
                        }
                    }
                    if pre_stack_length > &stack.len() {
                        terminal.clear()?;
                    }
                    result.clear();
                    memory.clear();
                    update_stack(&stack, &mut result, decimal_point);
                    update_memo(&memo_map, &mut memory);
                }
            }
        }

        if do_continue {
            ratatui::restore();
            calcrpn::print_help();
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            match input.trim() {
                "quit" | "q" => {
                    terminal.clear()?;
                    ratatui::restore();
                    break;
                }
                _ => terminal.clear()?,
            }
            *terminal = ratatui::init();
        } else {
            break;
        }
    }
    Ok(())
}

fn update_log(input_log: &mut VecDeque<String>, message: &mut String) {
    if input_log.len() > 8 {
        input_log.pop_front();
    }
    *message = input_log
        .iter()
        .rev()
        .fold("Hist: ".to_string(), |acc, x| acc + x + " → ");
}

fn update_memo(memo_map: &BTreeMap<String, CalcNum>, memory: &mut String) {
    // memo_mapをStringに変換しスペースで区切る
    for (key, sval) in memo_map.iter() {
        memory.push_str(&format!("{} -> {} ", key, sval.num_format(2)));
    }
}

fn update_stack(stack: &VecDeque<CalcNum>, result: &mut String, decimal_point: usize) {
    // 改行で区切る。windowsの場合は\r\n, Mac|linuxの場合は\n
    let sepalator = if cfg!(target_os = "windows") {
        "\r\n"
    } else {
        "\n"
    };
    *result = stack
        .iter()
        .rev()
        .take(10)
        .rev()
        .map(|x| x.num_format(decimal_point))
        .collect::<Vec<_>>()
        .join(sepalator);
}
