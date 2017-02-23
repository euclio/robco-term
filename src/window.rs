use ::game::*;
use ::ncurses::*;

use std::ascii::AsciiExt;
use std::char;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::mem;

use std::iter::repeat;
use ::itertools::Itertools;

use ::consts::*;

pub struct NCursesWindow;

pub fn create() -> Box<Window> {
    Box::new(NCursesWindow::new())
}

pub trait Window {
    fn render(&self, &GameState);
    fn handle_input(&self, &mut GameState) -> Option<InputEvent>;
}

impl NCursesWindow {
    fn new() -> NCursesWindow {
        // Enable all mouse events for the current terminal.
        env::set_var("TERM", "xterm-1003");

        setlocale(LcCategory::all, "");
        initscr();
        raw();

        // Extended keyboard and mouse events.
        keypad(stdscr(), true);
        nodelay(stdscr(), true);
        noecho();

        let mouse_events = ALL_MOUSE_EVENTS | REPORT_MOUSE_POSITION;
        mousemask(mouse_events as u32, None);
        mouseinterval(0);

        if has_mouse() {
            info!("Mouse driver initialized.")
        } else {
            info!("Error initializing mouse driver.");
        }

        NCursesWindow
    }
}

impl Drop for NCursesWindow {
    fn drop(&mut self) {
        refresh();
        endwin();
    }
}

impl Window for NCursesWindow {
    fn handle_input(&self, game_state: &mut GameState) -> Option<InputEvent> {
        let ch: i32 = getch();

        // Allow WASD and HJKL controls
        const KEY_W: i32 = 'w' as i32;
        const KEY_A: i32 = 'a' as i32;
        const KEY_S: i32 = 's' as i32;
        const KEY_D: i32 = 'd' as i32;

        const KEY_H: i32 = 'h' as i32;
        const KEY_J: i32 = 'j' as i32;
        const KEY_K: i32 = 'k' as i32;
        const KEY_L: i32 = 'l' as i32;

        const KEY_ESC: i32 = 27;
        const KEY_ENTER: i32 = '\n' as i32;
        match ch as i32 {
            KEY_LEFT | KEY_A | KEY_H => Some(InputEvent::Left),
            KEY_RIGHT | KEY_D | KEY_L => Some(InputEvent::Right),
            KEY_UP | KEY_W | KEY_K => Some(InputEvent::Up),
            KEY_DOWN | KEY_S | KEY_J => Some(InputEvent::Down),
            KEY_MOUSE => {
                let mut event: MEVENT = unsafe { mem::uninitialized() };
                assert!(getmouse(&mut event) == OK);

                game_state.cursor_position = (event.x, event.y);
                if event.bstate & (BUTTON1_PRESSED as u32) != 0 {
                    Some(InputEvent::Action)
                } else {
                    None
                }
            }
            KEY_ENTER => Some(InputEvent::Action),
            KEY_ESC => Some(InputEvent::Quit),
            _ => None,
        }
    }

    fn render(&self, game_state: &GameState) {
        refresh();
        erase();

        let starting_line = MARGIN + 5;

        // If the game is over, render the ending state and return early.
        if let Some(ref ending) = game_state.status {
            match *ending {
                GameEnding::Won => {
                    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
                    let reader = BufReader::new(File::open("resources/vault_boy.txt").unwrap());
                    let mut line_counter = 0;
                    for line in reader.lines().map(|l| l.unwrap()) {
                        mvprintw(line_counter as i32,
                                 0,
                                 &format!("{:^1$}", line, WINDOW_WIDTH as usize));
                        line_counter += 1;
                    }
                    mvprintw(line_counter as i32,
                             0,
                             &format!("{:^1$}", "ACCESS GRANTED", WINDOW_WIDTH as usize));
                }
                GameEnding::Lost => {
                    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
                    mvprintw((starting_line + ROWS) / 2,
                             0,
                             &format!("{:^1$}", "TERMINAL LOCKED", WINDOW_WIDTH as usize));
                    mvprintw((starting_line + ROWS + 1) / 2,
                             0,
                             &format!("{:^1$}",
                                      "PLEASE CONTACT AN ADMINISTRATOR",
                                      WINDOW_WIDTH as usize));
                }
            }
            return;
        }

        // Print information at top
        mvprintw(MARGIN, MARGIN, "ROBCO INDUSTRIES (TM) TERMLINK PROTOCOL");
        mvprintw(MARGIN + 1, MARGIN, "ENTER PASSWORD NOW");
        mvprintw(LINES() - 1, 0, "Press Esc to exit");

        // Print attempts remaining
        let visual_attempts = repeat("█")
            .take(game_state.attempts as usize)
            .join(" ");
        mvprintw(MARGIN + 3,
                 MARGIN,
                 &format!("{} ATTEMPT(S) LEFT: {}",
                          game_state.attempts,
                          visual_attempts));

        // Draw random addresses and word columns
        let highlight_positions = match game_state.get_entity_at_cursor() {
            Some(cursor_entity) => {
                if cursor_entity.highlighted() {
                    let (start, end) = cursor_entity.indices();
                    let start_x = start % WORD_COLUMN_WIDTH as usize;
                    let start_y = start / WORD_COLUMN_WIDTH as usize;
                    let end_x = end % WORD_COLUMN_WIDTH as usize;
                    let end_y = end / WORD_COLUMN_WIDTH as usize;
                    Some(((start_x, start_y), (end_x, end_y)))
                } else {
                    None
                }
            }
            None => None,
        };

        // Draw both columns
        for (column_index, column) in game_state.columns.iter().enumerate() {
            let word_data: Vec<char> = column.render_word_data().chars().collect::<Vec<char>>();
            let word_chunks = word_data.chunks(WORD_COLUMN_WIDTH as usize);
            for (line, (address, word_chunk)) in column.addresses
                .iter()
                .zip(word_chunks.into_iter())
                .enumerate() {
                let row = starting_line + line as i32;
                let col = MARGIN + column_index as i32 * (COLUMN_WIDTH + COLUMN_PADDING);
                let hex_address: String =
                    format!("{:#01$X}", address, ADDRESS_COLUMN_WIDTH as usize);
                let word_row: String = word_chunk.iter().map(|&c| c).collect::<String>();

                mvprintw(row, col, &(hex_address + " "));

                if let Some(((start_x, start_y), (end_x, end_y))) = highlight_positions {
                    if game_state.get_cursor_column_index().unwrap() != column_index {
                        // We're not in the correct column, so just write out the line and
                        // continue.
                        addstr(&word_row);
                        continue;
                    }

                    // If the highlight ends on the same line, we just iterate over the chunk and
                    // turn on and off the highlight at the start and the end.
                    if start_y == line && start_y == end_y {
                        for (i, c) in word_row.chars().enumerate() {
                            if i == start_x {
                                attron(A_STANDOUT());
                            }

                            if i == end_x {
                                attroff(A_STANDOUT());
                            }
                            addch(c as u32);
                        }
                    } else if start_y == line {
                        for (i, c) in word_row.chars().enumerate() {

                            if i == start_x {
                                attron(A_STANDOUT());
                            }
                            addch(c as u32);
                        }
                        attroff(A_STANDOUT());
                    } else if end_y == line {
                        attron(A_STANDOUT());
                        for (i, c) in word_row.chars().enumerate() {

                            if i == end_x {
                                attroff(A_STANDOUT());
                            }
                            addch(c as u32);
                        }
                    } else {
                        addstr(&word_row);
                    }
                } else {
                    addstr(&word_row);
                }
            }
        }

        // Draw the console.
        let console_entry = if let Some(entity) = game_state.get_entity_at_cursor() {
            match *entity {
                CursorEntity::Word { ref word, .. } => word.to_ascii_uppercase(),
                CursorEntity::Brackets { ref pair, .. } => pair.0.to_string(),
            }
        } else {

            // If we're in a column, display the character at the cursor. Otherwise, display an empty
            // string.
            match game_state.get_cursor_column_index() {
                Some(..) => {
                    let (x, y) = game_state.cursor_position;
                    char::from_u32(mvinch(y, x) as u32).unwrap().to_string()
                }
                None => "".to_string(),
            }
        };

        mvprintw(starting_line + ROWS - 1,
                 MARGIN + 2 * COLUMN_WIDTH + COLUMN_PADDING + MARGIN,
                 &format!(">{}", console_entry));

        // Draw the console entries, starting from the bottom.
        let mut entries_row = starting_line + ROWS - 3;
        for entry in game_state.entries.iter().rev() {
            let col = MARGIN + 2 * COLUMN_WIDTH + COLUMN_PADDING + MARGIN;

            // Only prints the lines if the entry would be within the address columns.
            let mvprintw_checked = |row, col, lines: &[&str]| {
                for (i, line) in lines.iter().rev().enumerate() {
                    if row >= starting_line {
                        mvprintw(row - i as i32, col, line);
                    }
                }
            };

            match *entry {
                Entry::Incorrect { num_correct, ref word } => {
                    mvprintw_checked(entries_row,
                                     col,
                                     &[&format!(">{}", word.to_ascii_uppercase()),
                                       ">Entry denied",
                                       &format!(">{}/{} correct.", num_correct, 7)]);
                }
                Entry::Correct { ref word } => {
                    mvprintw_checked(entries_row,
                                     col,
                                     &[&format!(">{}", word.to_ascii_uppercase()),
                                       ">Exact match!",
                                       ">Please wait",
                                       ">while system",
                                       ">is accessed."]);
                }
                Entry::DudRemoval => {
                    mvprintw_checked(entries_row, col, &[">", ">Dud removed."]);
                }
                Entry::AllowanceReplenish => {
                    mvprintw_checked(entries_row, col, &[">", ">Allowance", ">replenished."]);
                }
            }

            entries_row -= entry.display_rows() as i32;
        }

        // Move the cursor to the current position
        let (x, y) = game_state.cursor_position;
        mv(y, x);
    }
}
