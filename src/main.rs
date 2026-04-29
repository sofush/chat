use std::time::{Duration, Instant};

use color_eyre::Result;
use crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Position, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, List, ListItem, Paragraph};
use ratatui::{DefaultTerminal, Frame};
use tachyonfx::Interpolation;
use tachyonfx::{EffectManager, fx};

static BG: Color = Color::from_u32(0x1D2021);
static WIDGET_BG: Color = Color::from_u32(0x444444);
static WIDGET_FG: Color = Color::from_u32(0xdddddd);

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui::run(|terminal| App::new().run(terminal))
}

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: String,
    /// Position of cursor in the editor area.
    character_index: usize,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    messages: Vec<String>,
}

enum InputMode {
    Normal,
    Editing,
}

impl App {
    const fn new() -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            character_index: 0,
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;

        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete =
                self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input =
                before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn delete_word(&mut self) {
        while self
            .input
            .chars()
            .last()
            .is_some_and(|c| !c.is_whitespace())
        {
            self.delete_char();
        }

        self.delete_char();
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    const fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn submit_message(&mut self) {
        self.messages.push(self.input.clone());
        self.input.clear();
        self.reset_cursor();
    }

    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let mut effects: EffectManager<()> = EffectManager::default();
        let mut last_frame = Instant::now();

        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(3),
        ])
        .margin(2)
        .spacing(1);

        let [help_area, messages_area, input_area] =
            terminal.get_frame().area().layout(&layout);

        for area in [help_area, messages_area, input_area] {
            let timer = (1000, Interpolation::Linear);

            let effect = fx::fade_from(BG, BG, timer).with_area(area);

            effects.add_effect(effect);
        }

        loop {
            let elapsed = last_frame.elapsed();
            last_frame = Instant::now();

            terminal.draw(|frame| {
                self.render(
                    frame,
                    &mut effects,
                    elapsed,
                    help_area,
                    input_area,
                    messages_area,
                )
            })?;

            if event::poll(Duration::from_millis(16))?
                && let Some(key) = event::read()?.as_key_press_event()
            {
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('e') => {
                            self.input_mode = InputMode::Editing;
                        }
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        _ => {}
                    },
                    InputMode::Editing if key.kind == KeyEventKind::Press => {
                        if key.modifiers.contains(KeyModifiers::CONTROL)
                            && key.code == KeyCode::Char('w')
                        {
                            self.delete_word();
                            continue;
                        }

                        if key.modifiers.contains(KeyModifiers::CONTROL)
                            && key.code == KeyCode::Char('c')
                        {
                            self.input_mode = InputMode::Normal;
                        }

                        match key.code {
                            KeyCode::Enter => self.submit_message(),
                            KeyCode::Char(to_insert) => {
                                self.enter_char(to_insert)
                            }
                            KeyCode::Backspace => self.delete_char(),
                            KeyCode::Left => self.move_cursor_left(),
                            KeyCode::Right => self.move_cursor_right(),
                            KeyCode::Esc => self.input_mode = InputMode::Normal,
                            _ => {}
                        }
                    }
                    InputMode::Editing => {}
                }
            }
        }
    }

    fn render(
        &self,
        frame: &mut Frame,
        effects: &mut EffectManager<()>,
        elapsed: Duration,
        help_area: Rect,
        input_area: Rect,
        messages_area: Rect,
    ) {
        frame.render_widget(
            Block::default().style(Style::default().bg(BG)),
            frame.area(),
        );

        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " to exit, ".into(),
                    "e".bold(),
                    " to start editing.".bold(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Editing => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to send the message".into(),
                ],
                Style::default(),
            ),
        };
        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text).bg(WIDGET_BG);

        frame.render_widget(help_message, help_area);

        let input = Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default().bg(BG),
                InputMode::Editing => Style::default().fg(Color::Yellow).bg(BG),
            })
            .block(Block::default().title("Input"))
            .bg(WIDGET_BG)
            .fg(WIDGET_FG);

        frame.render_widget(input, input_area);

        match self.input_mode {
            InputMode::Normal => {}
            #[expect(clippy::cast_possible_truncation)]
            InputMode::Editing => frame.set_cursor_position(Position::new(
                input_area.x + self.character_index as u16,
                input_area.y + 1,
            )),
        }

        let messages: Vec<ListItem> = self
            .messages
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let content = Line::from(Span::raw(format!("{i}: {m}")));
                ListItem::new(content)
            })
            .collect();

        let messages = List::new(messages)
            .block(Block::default().title("Messages"))
            .bg(WIDGET_BG)
            .fg(WIDGET_FG);

        frame.render_widget(messages, messages_area);

        let area = frame.area();
        effects.process_effects(elapsed.into(), frame.buffer_mut(), area);
    }
}
