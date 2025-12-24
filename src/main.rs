mod wordlist;
use crate::wordlist::{VALIDLIST, WORDLIST};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use rand::{rng, seq::IndexedRandom};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Paragraph, Widget},
};
use std::{collections::HashMap, io};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut rng = rng();

    let word: Vec<char> = WORDLIST.choose(&mut rng).unwrap().chars().collect();
    let mut app = App {
        correct: word.try_into().unwrap(),
        guesses: Vec::with_capacity(6),
        typing: [None; 5],
        message: Line::from(""),
        exit: false,
        exit_message: None,
    };

    let app_result = app.run(&mut terminal);
    ratatui::restore();

    if let Ok(res) = app_result {
        if let Some(msg) = res {
            println!("{}", msg);
        }
    }

    Ok(())
}

#[derive(Debug, Default)]
pub struct App<'a> {
    correct: [char; 5],
    guesses: Vec<[char; 5]>,
    typing: [Option<char>; 5],
    message: Line<'a>,
    exit: bool,
    exit_message: Option<String>,
}

impl<'a> App<'a> {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<Option<String>> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(self.exit_message.clone())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc => {
                self.exit_message = Some(format!(
                    "The word was {} but you gave up. >:(",
                    self.correct.iter().collect::<String>()
                ));
                self.exit = true;
            }
            KeyCode::Char(char) => {
                self.message = Line::from("");

                let smol = char.to_ascii_uppercase();

                if smol.is_ascii_uppercase() {
                    for key in self.typing.iter_mut() {
                        if key.is_none() {
                            *key = Some(smol);
                            break;
                        }
                    }
                }

                if !self.typing.contains(&None) {
                    let word: String = self.typing.iter().filter_map(|c| c.as_ref()).collect();

                    if !VALIDLIST.contains(&word.as_str()) {
                        return self.message = Line::from("not a valid word!".red());
                    }
                }
            }
            KeyCode::Backspace => {
                self.message = Line::from("");
                self.typing.reverse();

                for key in self.typing.iter_mut() {
                    if key.is_some() {
                        *key = None;
                        break;
                    }
                }

                self.typing.reverse();
            }
            KeyCode::Enter => {
                if self.typing.contains(&None) {
                    return self.message = Line::from("5 chars needed!".red());
                }

                let word: String = self.typing.iter().filter_map(|c| c.as_ref()).collect();

                if !VALIDLIST.contains(&word.as_str()) {
                    return self.message = Line::from("not a valid word!".red());
                }

                // is valid word pog
                let correct = self.correct.iter().collect::<String>();
                if word == correct {
                    self.exit_message = Some(format!(
                        "Congrats! The word was {}. You guessed it in {} tries.",
                        correct,
                        self.guesses.len() + 1
                    ));
                    self.exit = true;

                    return self.message = Line::from("you won!!".green());
                }

                self.message = Line::from("errr! try again".red());
                let guess: [char; 5] = word.chars().collect::<Vec<_>>().try_into().unwrap();
                self.guesses.push(guess);
                self.typing = [None; 5];

                if self.guesses.len() >= 6 {
                    self.exit = true;
                    self.exit_message =
                        Some(format!("You lost after 6 tries! The word was {}.", correct));
                }
            }
            _ => {}
        }
    }
}

impl<'a> Widget for &'a App<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" W O R D L E ".bold());
        let instructions = Line::from(vec![
            " Press a ".into(),
            "<key>".blue().bold(),
            " Confirm ".into(),
            "<ENTER>".blue().bold(),
            " Quit ".into(),
            "<ESC> ".blue().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let inner = block.inner(area);
        block.render(area, buf);

        let cell_width: u16 = 7;
        let row_height: u16 = 3;
        let total_cells_width: u16 = cell_width * 5;

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Max(1),
                Constraint::Length(row_height),
                Constraint::Length(row_height),
                Constraint::Length(row_height),
                Constraint::Length(row_height),
                Constraint::Length(row_height),
                Constraint::Length(row_height),
            ])
            .split(inner);

        let message_block = Block::default();
        let paragraph_block = Paragraph::new(self.message.clone())
            .centered()
            .block(message_block);
        paragraph_block.render(rows[0], buf);

        let mut typing_showed = false;

        for i in 2..8 {
            let centered = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Min(0),
                    Constraint::Length(total_cells_width),
                    Constraint::Min(0),
                ])
                .split(rows[i]);

            let cells = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(cell_width); 5])
                .split(centered[1]);

            let guess: Option<[char; 5]> = if self.guesses.len() > i - 2 {
                Some(self.guesses[i - 2])
            } else {
                None
            };

            if let Some(word) = guess {
                let mut done: HashMap<char, usize> = HashMap::new();

                for j in 0..5 {
                    let cell_block;

                    if word[j] == self.correct[j] {
                        cell_block = Block::bordered().green();

                        if let Some(count) = done.get_mut(&word[j]) {
                            *count = count.saturating_add(1);
                        } else {
                            done.insert(word[j], 1usize);
                        }
                    } else {
                        let mut count = self.correct.iter().filter(|x| x == &&word[j]).count();

                        if let Some(has) = done.get(&word[j]) {
                            count = count.saturating_sub(*has);
                        }

                        cell_block = if count > 0 {
                            done.entry(word[j]).and_modify(|e| *e += 1).or_insert(1);
                            Block::bordered().yellow()
                        } else {
                            Block::bordered().red()
                        };
                    }

                    let paragraph = Paragraph::new(word[j].to_string())
                        .centered()
                        .block(cell_block);
                    paragraph.render(cells[j], buf);
                }
            } else {
                for j in 0..5 {
                    if !typing_showed && let Some(c) = self.typing[j] {
                        let cell_block = Block::bordered().blue();
                        let paragraph = Paragraph::new(c.to_string()).centered().block(cell_block);
                        paragraph.render(cells[j], buf);
                    } else {
                        if !typing_showed {
                            Block::bordered().light_blue().render(cells[j], buf);
                        } else {
                            Block::bordered().render(cells[j], buf);
                        }
                    }
                }

                if !typing_showed {
                    typing_showed = true
                }
            }
        }
    }
}
