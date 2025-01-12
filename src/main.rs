mod calcenums;
use calcenums::{manage_stack, DegMode};

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{collections::VecDeque, io};

const RESULT_DISPLAY_LENGTH: u16 = 12;

fn calculate_rpn(
    expression: &str,
    stack: &mut VecDeque<f64>,
    degmode: &mut DegMode,
) -> Result<String, String> {
    manage_stack(expression, stack, degmode)?;
    let mut result = "".to_string();
    // 改行で区切る。windowsの場合は\r\n, Mac|linuxの場合は\n
    let sepalator = if cfg!(target_os = "windows") {
        "\r\n"
    } else {
        "\n"
    };
    // stackをStringに変換し要素毎に改行を入れる
    let length = stack.len();
    for (i, val) in stack.iter().enumerate() {
        result.push_str(&format!("{}: {:.4}", length - i, val));
        if i < stack.len() - 1 {
            result.push_str(sepalator);
        }
    }
    Ok(result)
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut stack: VecDeque<f64> = VecDeque::with_capacity(16);
    let mut degmode = DegMode::Deg;
    let mut input = String::new();
    let mut result = String::new();
    let mut message = String::new();
    let sepalator = if cfg!(target_os = "windows") {
        "\r\n"
    } else {
        "\n"
    };

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(5),
                        Constraint::Length(RESULT_DISPLAY_LENGTH),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(f.size());
            let input_block = Block::default().title("Input ").borders(Borders::TOP);
            let input_text = Paragraph::new(input.as_ref()).block(input_block);
            f.render_widget(input_text, chunks[2]);

            let result_block = Block::default().title("Result").borders(Borders::ALL);
            let result_text = Paragraph::new(result.as_ref()).block(result_block);
            f.render_widget(result_text, chunks[1]);

            let help_block = Block::default().title("Message").borders(Borders::ALL);
            let help_text = Paragraph::new(format!(
                "{:?}{}Enter: Calc | Esc : Quit{}{}",
                degmode, sepalator, sepalator, message
            ))
            .block(help_block);
            f.render_widget(help_text, chunks[0]);
        })?;
        let input_col = 1;
        let input_row = RESULT_DISPLAY_LENGTH + 7;
        let cursor_col = input_col + input.len() as u16;
        execute!(
            terminal.backend_mut(),
            crossterm::cursor::MoveTo(cursor_col, input_row),
            crossterm::cursor::Show
        )?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char(c) => {
                    input.push(c);
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Enter => {
                    match calculate_rpn(&input, &mut stack, &mut degmode) {
                        Ok(value) => {
                            result = value;
                            message.clear();
                        }
                        Err(e) => {
                            message = format!("Error: {}", e);
                        }
                    }
                    input.clear();
                }
                KeyCode::Esc => {
                    break;
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
