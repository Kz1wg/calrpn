// 画面描画
use crate::app::{App, Message, Mode};
use crate::calcrpn::{DegMode, Memorize, STACK_SIZE};
use crate::commands;
use crate::highlight::{self, TokenKind};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Clear, Paragraph},
};

const PROMPT: &str = "❯ ";

// 配色 (256色パレット。vim-airline の dark テーマに近い色)
// 画面全体の色はここで一元管理する
mod palette {
    use ratatui::style::Color;
    // ステータスバー
    pub const FILL: Color = Color::Indexed(234);
    pub const SECTION_B: Color = Color::Indexed(238);
    pub const SECTION_C: Color = Color::Indexed(236);
    pub const DARK_TEXT: Color = Color::Indexed(235);
    pub const NORMAL: Color = Color::Indexed(149); // 緑
    pub const WAITING: Color = Color::Indexed(214); // 橙
    pub const ERROR: Color = Color::Indexed(167); // 赤
    pub const HELP: Color = Color::Indexed(110); // 青
    // 共通
    pub const TEXT: Color = Color::Indexed(252);
    pub const DIM: Color = Color::Indexed(244);
    pub const BORDER: Color = Color::Indexed(239);
    pub const TITLE: Color = Color::Indexed(250);
    pub const ACCENT: Color = Color::Indexed(110);
    pub const X_ROW: Color = Color::Indexed(236); // スタックの X 行の背景
    // シンタックスハイライト
    pub const OPERATOR: Color = Color::Indexed(214);
    pub const FUNCTION: Color = Color::Indexed(110);
    pub const STACK_CMD: Color = Color::Indexed(176);
    pub const MEMORY_CMD: Color = Color::Indexed(180);
    pub const CONSTANT: Color = Color::Indexed(116);
    pub const KEY: Color = Color::Indexed(149);
}

pub fn render(frame: &mut Frame, app: &App) {
    // スタックの表示行数 (空のときも1行確保して "(empty)" を表示)
    let stack_rows = app.state.stack.len().clamp(1, STACK_SIZE) as u16;
    // ステータスバーは余白なしで全幅に表示する
    let [status, body] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(frame.area());
    let [memory, stack, input, message, footer, _] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(stack_rows + 2),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .margin(1)
    .areas(body);

    render_status(frame, app, status);
    render_memory(frame, app, memory);
    render_stack(frame, app, stack);
    render_input(frame, app, input);
    render_message(frame, app, message);
    // キー操作の一覧は入力欄が空のときだけ表示する
    if app.input.text().is_empty() {
        render_footer(frame, footer);
    }

    if let Mode::Help { scroll } = app.mode {
        render_help(frame, app, scroll);
    }
}

// 角丸・控えめな枠線のパネル
fn panel(title: &str) -> Block<'_> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(palette::BORDER))
        .title(Span::styled(format!(" {title} "), Style::new().fg(palette::TITLE)))
}

fn dim(text: impl Into<String>) -> Span<'static> {
    Span::styled(text.into(), Style::new().fg(palette::DIM))
}

// ---- Airline 風ステータスバー ----

// Powerline 対応フォント (Nerd Font 等) がない環境では false にする
const POWERLINE_GLYPHS: bool = true;
// 左セグメントの区切り / 右セグメントの区切り
const SEP_LEFT: &str = if POWERLINE_GLYPHS { "\u{e0b0}" } else { "" };
const SEP_RIGHT: &str = if POWERLINE_GLYPHS { "\u{e0b2}" } else { "" };

struct Segment {
    text: String,
    fg: Color,
    bg: Color,
    bold: bool,
}

impl Segment {
    fn new(text: impl Into<String>, fg: Color, bg: Color) -> Self {
        Self {
            text: text.into(),
            fg,
            bg,
            bold: false,
        }
    }

    fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    fn span(&self) -> Span<'static> {
        let mut style = Style::new().fg(self.fg).bg(self.bg);
        if self.bold {
            style = style.add_modifier(Modifier::BOLD);
        }
        Span::styled(format!(" {} ", self.text), style)
    }
}

// 左寄せセグメント列: [seg1]▶[seg2]▶ ... ▶(地)
fn powerline_left(segments: &[Segment], end_bg: Color) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for (i, seg) in segments.iter().enumerate() {
        let next_bg = segments.get(i + 1).map_or(end_bg, |s| s.bg);
        spans.push(seg.span());
        spans.push(Span::styled(SEP_LEFT, Style::new().fg(seg.bg).bg(next_bg)));
    }
    spans
}

// 右寄せセグメント列: (地)◀[seg1]◀[seg2] ...
fn powerline_right(segments: &[Segment], start_bg: Color) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut prev_bg = start_bg;
    for seg in segments {
        spans.push(Span::styled(SEP_RIGHT, Style::new().fg(seg.bg).bg(prev_bg)));
        spans.push(seg.span());
        prev_bg = seg.bg;
    }
    spans
}

// 現在のモード表示
fn mode_segment(app: &App) -> Segment {
    let (label, bg) = if matches!(app.mode, Mode::Help { .. }) {
        ("HELP", palette::HELP)
    } else if let Some(label) = memo_wait_label(&app.memo_mode) {
        (label, palette::WAITING)
    } else if matches!(app.message, Message::Error(_)) {
        ("ERR", palette::ERROR)
    } else {
        ("RPN", palette::NORMAL)
    };
    Segment::new(label, palette::DARK_TEXT, bg).bold()
}

fn render_status(frame: &mut Frame, app: &App, area: Rect) {
    let deg = match app.state.degmode {
        DegMode::Deg => "DEG",
        DegMode::Rad => "RAD",
    };
    let left = powerline_left(
        &[
            mode_segment(app),
            Segment::new(deg, palette::TEXT, palette::SECTION_B),
            Segment::new(
                format!("{} {}", app.format.notation.label(), app.format.decimals),
                palette::TEXT,
                palette::SECTION_C,
            ),
        ],
        palette::FILL,
    );
    let right = powerline_right(
        &[
            Segment::new(format!("UNDO {}", app.undo_count()), palette::TEXT, palette::SECTION_C),
            Segment::new(
                format!("MEM {}", app.state.memo_map.len()),
                palette::TEXT,
                palette::SECTION_B,
            ),
            Segment::new(
                format!("STK {}/{}", app.state.stack.len(), STACK_SIZE),
                palette::DARK_TEXT,
                palette::NORMAL,
            )
            .bold(),
        ],
        palette::FILL,
    );

    let right = Line::from(right);
    let [left_area, right_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(right.width() as u16)])
            .areas(area);
    let fill = Style::new().bg(palette::FILL);
    frame.render_widget(Line::from(left).style(fill), left_area);
    frame.render_widget(right.style(fill), right_area);
}

// キー待ち中の表示
fn memo_wait_label(memo_mode: &Option<Memorize>) -> Option<&'static str> {
    match memo_mode {
        Some(Memorize::Store(None)) => Some("STO ▸ key?"),
        Some(Memorize::Recall(None)) => Some("RCL ▸ key?"),
        Some(Memorize::Delete(None)) => Some("MDEL ▸ key?"),
        Some(Memorize::Tvm(None)) => Some("TVM ▸ n/iyr/pv/pmt/fv?"),
        _ => None,
    }
}

// ---- 本体 ----

fn render_memory(frame: &mut Frame, app: &App, area: Rect) {
    let mut spans = vec![dim(" Memory  ")];
    if app.state.memo_map.is_empty() {
        spans.push(dim("(none)"));
    }
    for (key, val) in &app.state.memo_map {
        spans.push(Span::styled(key.clone(), Style::new().fg(palette::KEY)));
        spans.push(dim("="));
        spans.push(Span::styled(
            format!("{}   ", val.num_format(&app.format)),
            Style::new().fg(palette::TEXT),
        ));
    }
    frame.render_widget(Line::from(spans), area);
}

fn render_stack(frame: &mut Frame, app: &App, area: Rect) {
    let block = panel("Stack");
    let inner_width = block.inner(area).width as usize;

    let len = app.state.stack.len();
    if len == 0 {
        frame.render_widget(Paragraph::new(Line::from(dim(" (empty)"))).block(block), area);
        return;
    }

    // 値を番号のすぐ右に置き、全行で小数点の位置を揃える
    // (整数部は右寄せ・小数部は左寄せ。視線の左右移動を減らすため右端寄せにはしない)
    let values: Vec<String> = app
        .state
        .stack
        .iter()
        .map(|v| v.num_format(&app.format))
        .collect();
    let int_width = values.iter().map(|v| split_decimal(v).0.len()).max().unwrap_or(0);
    let frac_width = values.iter().map(|v| split_decimal(v).1.len()).max().unwrap_or(0);

    // 上が古い値、最下段がレベル1 (X)
    let lines: Vec<Line> = values
        .iter()
        .enumerate()
        .map(|(i, value)| {
            let level = len - i;
            let (int_part, frac_part) = split_decimal(value);
            let label = format!("{level:>3}  ");
            let number = format!("{int_part:>int_width$}{frac_part:<frac_width$}");
            if level == 1 {
                // X 行は背景色を行全体に敷いて強調する
                let bg = Style::new().bg(palette::X_ROW);
                let pad = inner_width.saturating_sub(label.len() + number.len());
                Line::from(vec![
                    Span::styled(label, bg.fg(palette::ACCENT)),
                    Span::styled(number, bg.fg(palette::TEXT).add_modifier(Modifier::BOLD)),
                    Span::styled(" ".repeat(pad), bg),
                ])
            } else {
                Line::from(vec![
                    Span::styled(label, Style::new().fg(palette::DIM)),
                    Span::styled(number, Style::new().fg(palette::TEXT)),
                ])
            }
        })
        .collect();
    frame.render_widget(Paragraph::new(lines).block(block), area);
}

// 数値文字列を最初の小数点の前後で分ける ("12.50" -> ("12", ".50"), "3" -> ("3", ""))
// 複素数 "1.000 + 2.000i" は実部の小数点で分かれる
fn split_decimal(value: &str) -> (&str, &str) {
    match value.find('.') {
        Some(i) => value.split_at(i),
        None => (value, ""),
    }
}

fn token_style(kind: TokenKind) -> Style {
    let style = Style::new();
    match kind {
        TokenKind::Space | TokenKind::Number => style.fg(palette::TEXT),
        TokenKind::Operator => style.fg(palette::OPERATOR).add_modifier(Modifier::BOLD),
        TokenKind::Function => style.fg(palette::FUNCTION),
        TokenKind::Stack => style.fg(palette::STACK_CMD),
        TokenKind::Memory => style.fg(palette::MEMORY_CMD),
        TokenKind::Constant => style.fg(palette::CONSTANT),
        TokenKind::Key => style.fg(palette::KEY),
        TokenKind::Unknown => style
            .fg(palette::ERROR)
            .add_modifier(Modifier::UNDERLINED),
    }
}

fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let block = panel("Input");
    let inner = block.inner(area);

    let mut spans = vec![Span::styled(PROMPT, Style::new().fg(palette::ACCENT))];
    for (text, kind) in highlight::classify(app.input.text(), &app.state.memo_map, &app.memo_mode) {
        spans.push(Span::styled(text, token_style(kind)));
    }
    // 補完候補の残りを薄く表示 (Tab / → で確定)
    if let Some(ghost) = app.ghost() {
        spans.push(Span::styled(ghost, Style::new().fg(palette::BORDER)));
    }
    frame.render_widget(Paragraph::new(Line::from(spans)).block(block), area);

    if matches!(app.mode, Mode::Normal) {
        // 全角文字なども考慮して表示幅でカーソル位置を求める
        let offset = Span::raw(PROMPT).width() + Span::raw(app.input.before_cursor()).width();
        let max_x = inner.right().saturating_sub(1);
        let x = (inner.x as usize + offset).min(max_x as usize) as u16;
        frame.set_cursor_position((x, inner.y));
    }
}

// メッセージがあれば表示し、なければ直近の入力履歴を表示する
fn render_message(frame: &mut Frame, app: &App, area: Rect) {
    let line = match &app.message {
        Message::Error(m) => Line::from(vec![
            Span::styled(" ✗ ", Style::new().fg(palette::ERROR).add_modifier(Modifier::BOLD)),
            Span::styled(m.clone(), Style::new().fg(palette::ERROR)),
        ]),
        Message::Info(m) => Line::from(vec![
            Span::styled(" ✓ ", Style::new().fg(palette::ACCENT)),
            Span::styled(m.clone(), Style::new().fg(palette::ACCENT)),
        ]),
        Message::Hint(m) => Line::from(vec![
            Span::styled(" ⇥ ", Style::new().fg(palette::DIM)),
            dim(m.clone()),
        ]),
        Message::None => {
            // 新しい入力を先頭に表示
            let hist = app
                .input_log
                .iter()
                .rev()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join("  ←  ");
            Line::from(vec![dim(" Hist  "), dim(hist)])
        }
    };
    frame.render_widget(line, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let hints = [
        ("Enter", "calc"),
        ("↑↓", "history"),
        ("Tab", "complete"),
        ("Ctrl-Z", "undo"),
        ("F1", "help"),
        ("Ctrl-C", "quit"),
    ];
    let mut spans = vec![Span::raw(" ")];
    for (key, desc) in hints {
        spans.push(Span::styled(key, Style::new().fg(palette::ACCENT)));
        spans.push(dim(format!(" {desc}   ")));
    }
    frame.render_widget(Line::from(spans), area);
}

fn render_help(frame: &mut Frame, app: &App, scroll: u16) {
    let area = centered(frame.area(), 90, 90);
    let block = panel("Help").title_bottom(Line::from(dim(" ↑↓/PgUp/PgDn: scroll  Esc: close ")));
    let inner_height = block.inner(area).height;

    let lines: Vec<Line> = commands::help_text()
        .lines()
        .map(|l| {
            if l.starts_with('[') {
                Line::styled(
                    l.to_string(),
                    Style::new().fg(palette::WAITING).add_modifier(Modifier::BOLD),
                )
            } else {
                Line::styled(l.to_string(), Style::new().fg(palette::TEXT))
            }
        })
        .collect();
    // 末尾を越えてスクロールしないよう制限
    let max_scroll = (lines.len() as u16).saturating_sub(inner_height);
    app.help_max_scroll.set(max_scroll);

    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .scroll((scroll.min(max_scroll), 0)),
        area,
    );
}

// area の中央に、幅・高さをパーセント指定した領域を返す
fn centered(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let width = (area.width as u32 * percent_x as u32 / 100) as u16;
    let height = (area.height as u32 * percent_y as u32 / 100) as u16;
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::split_decimal;

    #[test]
    fn split_at_decimal_point() {
        assert_eq!(split_decimal("12.50"), ("12", ".50"));
        assert_eq!(split_decimal("-3"), ("-3", ""));
        assert_eq!(split_decimal("1.000 + 2.000i"), ("1", ".000 + 2.000i"));
    }
}
