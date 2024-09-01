use crate::fuzzy::{FuzzyFinder, FuzzyItem};
use crate::config::UIConfig;
use crate::error::FuzzydError;
use unicode_width::UnicodeWidthStr;
use unicode_width::UnicodeWidthChar;
use std::io::{Write, stdout};
use termion::input::TermRead;
use termion::event::Key;
use termion::{cursor, clear, color, style};
use termion::screen::IntoAlternateScreen;
use termion::raw::IntoRawMode;

pub struct TerminalUI {
    config: UIConfig,
    debug: bool,
}

impl TerminalUI {
    pub fn new(config: UIConfig, debug: bool) -> Self {
        TerminalUI { config, debug }
    }

    pub fn run(&mut self, finder: &mut FuzzyFinder) -> Result<Option<FuzzyItem>, FuzzydError> {
        let mut screen = stdout().into_raw_mode().unwrap().into_alternate_screen().unwrap();
        let mut query = String::new();
        let mut cursor_pos = 0;
        let mut selected = 0;
        let mut history: Vec<String> = Vec::new();
        let mut history_index = 0;

        loop {
            let matches = finder.find(&query);
            self.draw_screen(&mut screen, &query, cursor_pos, &matches, selected)?;

            match std::io::stdin().keys().next().unwrap()? {
                Key::Char('\n') if !matches.is_empty() => {
                    let selected_item = matches[selected].1.clone();
                    history.push(query.clone());
                    return Ok(Some(selected_item));
                },
                Key::Char(c) => {
                    query.insert(cursor_pos, c);
                    cursor_pos += 1;
                    selected = 0;
                },
                Key::Backspace if cursor_pos > 0 => {
                    query.remove(cursor_pos - 1);
                    cursor_pos -= 1;
                    selected = 0;
                },
                Key::Left if cursor_pos > 0 => cursor_pos -= 1,
                Key::Right if cursor_pos < query.len() => cursor_pos += 1,
                Key::Down if selected > 0 => selected -= 1,
                Key::Up if selected < matches.len().saturating_sub(1) => selected += 1,
                Key::Ctrl('a') => cursor_pos = 0,
                Key::Ctrl('e') => cursor_pos = query.len(),
                Key::Ctrl('u') => {
                    query.drain(..cursor_pos);
                    cursor_pos = 0;
                    selected = 0;
                },
                Key::Ctrl('k') => {
                    query.truncate(cursor_pos);
                    selected = 0;
                },
                Key::Ctrl('w') => {
                    while cursor_pos > 0 && !query.chars().nth(cursor_pos - 1).unwrap().is_whitespace() {
                        query.remove(cursor_pos - 1);
                        cursor_pos -= 1;
                    }
                    selected = 0;
                },
                Key::Ctrl('p') if !history.is_empty() => {
                    if history_index < history.len() {
                        history_index += 1;
                        query = history[history.len() - history_index].clone();
                        cursor_pos = query.len();
                        selected = 0;
                    }
                },
                Key::Ctrl('n') if history_index > 0 => {
                    history_index -= 1;
                    if history_index == 0 {
                        query.clear();
                    } else {
                        query = history[history.len() - history_index].clone();
                    }
                    cursor_pos = query.len();
                    selected = 0;
                },
                Key::Esc => {
                    if query.is_empty() {
                        return Ok(None);
                    } else {
                        query.clear();
                        cursor_pos = 0;
                        selected = 0;
                    }
                },
                Key::Ctrl('c') => {
                    return Err(FuzzydError::UserInterrupt)
                },
                _ => {}
            }
        }
    }

    fn draw_screen<W: Write>(&self, screen: &mut W, query: &str, cursor_pos: usize, matches: &[(f64, &FuzzyItem)], selected: usize) -> Result<(), FuzzydError> {
        write!(screen, "{}", clear::All)?;

        let (width, height) = termion::terminal_size()?;
        let max_items = height.saturating_sub(if self.debug { 4 } else { 3 }) as usize; // Adjust based on debug mode

        // Display selected item details at the top
        if !matches.is_empty() {
            let selected_item = &matches[selected].1;
            let display_text = if self.debug {
                format!("{} ({})", selected_item.display, selected_item.exec)
            } else {
                selected_item.display.to_string()
            };
            write!(screen, "{}{}{}{}", 
                cursor::Goto(1, 1),
                color::Fg(color::Green),
                style::Bold,
                truncate_str(&display_text, width as usize)
            )?;
            write!(screen, "{}", style::Reset)?;

            // Always display description on the second line
            write!(screen, "{}{}",
                cursor::Goto(1, 2),
                truncate_str(&selected_item.description, width as usize)
            )?;
        } else {
            // If there are no matches, still reserve the first two lines
            write!(screen, "{}\n\n", cursor::Goto(1, 1))?;
        }

        // Calculate the start index to ensure the selected item is visible
        let start_index = if matches.len() > max_items {
            if selected >= max_items {
                selected.saturating_sub(max_items - 1)
            } else {
                0
            }
        } else {
            0
        };

        // Display matches
        for (i, (score, item)) in matches.iter().enumerate().skip(start_index).take(max_items) {
            let y = (height - if self.debug { 1 } else { 0 } - (i - start_index) as u16).saturating_sub(1);
            let is_selected = i == selected;
            let display = highlight_matches(&item.display, query, width as usize - 15, is_selected);
            if is_selected {
                write!(screen, "{}{}> {} {}", 
                    cursor::Goto(1, y),
                    color::Fg(color::Green),
                    item.icon,
                    display
                )?;
                if self.debug {
                    write!(screen, " ({:.1}) ({})", score, item.source_path)?;
                }
                write!(screen, "{}", color::Fg(color::Reset))?;
            } else {
                write!(screen, "{}  {} {}", cursor::Goto(1, y), item.icon, display)?;
                if self.debug {
                    write!(screen, " ({:.1}) ({})", score, item.source_path)?;
                }
            }
            write!(screen, "{}", style::Reset)?; // Ensure styles are reset after each line
        }

        // Display debug info only if debug mode is enabled
        if self.debug {
            write!(screen, "{}Matches: {} | Terminal size: {}x{} | Selected: {} | Cursor: {} | Start: {}", 
                cursor::Goto(1, height), 
                matches.len(),
                width,
                height,
                selected,
                cursor_pos,
                start_index
            )?;
        }

        // Display search box
        let search_y = if self.debug { height - 1 } else { height };
        write!(screen, "{}{}{}{} {}{}", 
            cursor::Goto(1, search_y),
            color::Fg(color::White),
            style::Bold,
            self.config.prompt.as_deref().unwrap_or("#"),
            style::Reset,
            query
        )?;
        
        // Position the cursor on the search box line
        write!(screen, "{}", cursor::Goto((cursor_pos + 3) as u16, search_y))?;

        screen.flush()?;
        Ok(())
    }
}

fn highlight_matches(text: &str, query: &str, max_width: usize, is_selected: bool) -> String {
    let mut result = String::new();
    let lowercase_text = text.to_lowercase();
    let lowercase_query: Vec<char> = query.to_lowercase().chars().collect();

    let mut matched_indices = vec![false; text.len()];

    // Find all matching character indices
    for &q_char in &lowercase_query {
        for (i, t_char) in lowercase_text.char_indices() {
            if q_char == t_char {
                matched_indices[i] = true;
            }
        }
    }

    let mut is_highlighting = false;
    for (i, c) in text.char_indices() {
        if matched_indices[i] {
            if !is_highlighting {
                if is_selected {
                    result.push_str(&format!("{}{}", style::Bold, color::Fg(color::Green)));
                } else {
                    result.push_str(&style::Bold.to_string());
                }
                is_highlighting = true;
            }
        } else if is_highlighting {
            result.push_str(&style::Reset.to_string());
            is_highlighting = false;
        }
        result.push(c);
    }

    if is_highlighting {
        result.push_str(&style::Reset.to_string());
    }

    truncate_str(&result, max_width)
}

fn truncate_str(s: &str, max_width: usize) -> String {
    if s.width() <= max_width {
        s.to_string()
    } else {
        let mut width = 0;
        let mut truncated = String::new();
        for c in s.chars() {
            let char_width = c.width().unwrap_or(0);
            if width + char_width > max_width.saturating_sub(1) {
                break;
            }
            width += char_width;
            truncated.push(c);
        }
        truncated.push('…');
        truncated
    }
}