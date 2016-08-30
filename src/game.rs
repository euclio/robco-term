use std::ascii::AsciiExt;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

use ::itertools::Itertools;
use ::rand::{self, Rng};
use ::rand::distributions::{IndependentSample, Range};
use ::time::Duration;

use ::consts::*;

pub enum InputEvent {
    Up,
    Down,
    Left,
    Right,
    Action,
    Quit,
}

pub enum Entry {
    Correct {
        word: String,
    },
    Incorrect {
        word: String,
        num_correct: i32,
    },
    DudRemoval,
    AllowanceReplenish,
}

impl Entry {
    pub fn display_rows(&self) -> usize {
        use self::Entry::*;
        match *self {
            Correct { .. } => 5,
            Incorrect { .. } => 3,
            DudRemoval => 2,
            AllowanceReplenish => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub enum CursorEntity {
    Word {
        word: String,
        guessed: bool,
        index: usize,
        removed: bool,
    },
    Brackets {
        pair: (char, char),
        consumed: bool,
        indices: (usize, usize),
    },
}

impl CursorEntity {
    pub fn indices(&self) -> (usize, usize) {
        match *self {
            CursorEntity::Word { ref word, index, .. } => (index, index + word.len()),
            CursorEntity::Brackets { indices, .. } => (indices.0, indices.1 + 1),
        }
    }

    pub fn highlighted(&self) -> bool {
        match *self {
            CursorEntity::Word { guessed, .. } => !guessed,
            CursorEntity::Brackets { consumed, .. } => !consumed,
        }
    }
}

pub enum GameEnding {
    Won,
    Lost,
}

pub struct GameState {
    pub attempts: i32,
    pub columns: [Column; COLUMNS as usize],
    pub cursor_position: (i32, i32),
    pub is_playing: bool,
    pub correct_word: String,
    pub entries: Vec<Entry>,
    pub status: Option<GameEnding>,
}

impl GameState {
    pub fn update(&mut self, event: Option<InputEvent>, elapsed_time: Duration) {
        if let Some(event) = event {
            match event {
                InputEvent::Left => self.cursor_position.0 -= 1,
                InputEvent::Right => self.cursor_position.0 += 1,
                InputEvent::Up => self.cursor_position.1 -= 1,
                InputEvent::Down => self.cursor_position.1 += 1,
                InputEvent::Quit => self.is_playing = false,
                InputEvent::Action => self.select_entity(),
            }
        }

        // TODO: Animations
        let _ = elapsed_time;
    }

    pub fn get_cursor_column_index(&self) -> Option<usize> {
        let (x, y) = self.cursor_position;

        // Check that the cursor is actually inside one of the columns.
        if y < COLUMN_START_ROW || y > COLUMN_END_ROW {
            return None;
        }

        let left_col_start = MARGIN + ADDRESS_COLUMN_WIDTH + INNER_COLUMN_PADDING;
        let left_col_end = left_col_start + WORD_COLUMN_WIDTH;
        let right_col_start = left_col_end + COLUMN_PADDING + ADDRESS_COLUMN_WIDTH +
                              INNER_COLUMN_PADDING;
        let right_col_end = right_col_start + COLUMN_WIDTH;

        // Check if it's in the column we asked for.
        if left_col_start <= x && x < left_col_end {
            return Some(0);
        } else if right_col_start <= x && x < right_col_end {
            return Some(1);
        }

        None
    }

    fn column_coordinates(&self) -> Option<(usize, usize)> {
        let (x, y) = self.cursor_position;

        let left_col_start = MARGIN + ADDRESS_COLUMN_WIDTH + INNER_COLUMN_PADDING;
        let left_col_end = left_col_start + WORD_COLUMN_WIDTH;
        let right_col_start = left_col_end + COLUMN_PADDING + ADDRESS_COLUMN_WIDTH +
                              INNER_COLUMN_PADDING;

        if let Some(column_index) = self.get_cursor_column_index() {
            // We know that the cursor is inside a column. Now we need to determine where it is in
            // relation to word and bracket pairs.
            let col_x: i32 = if column_index == 0 {
                x - left_col_start
            } else if column_index == 1 {
                x - right_col_start
            } else {
                panic!();
            };
            let col_y = y - COLUMN_START_ROW;

            // Translate those coordinates into an index into the column data.
            let index: usize = col_y as usize * (WORD_COLUMN_WIDTH as usize) + col_x as usize;
            Some((column_index, index))
        } else {
            None
        }
    }

    pub fn get_entity_at_cursor(&self) -> Option<&CursorEntity> {
        if let Some(cursor_position) = self.column_coordinates() {
            let (column_index, index) = cursor_position as (usize, usize);
            for entity in &self.columns[column_index].entities {
                let (start, end) = entity.indices();
                match *entity {
                    CursorEntity::Word { removed, .. } => {
                        if start <= index && index < end {
                            if !removed {
                                return Some(entity);
                            } else {
                                return None;
                            }
                        }
                    }
                    CursorEntity::Brackets { consumed, .. } => {
                        if start == index {
                            if !consumed {
                                return Some(entity);
                            } else {
                                return None;
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn get_entity_at_cursor_mut(&mut self) -> Option<&mut CursorEntity> {
        if let Some(cursor_position) = self.column_coordinates() {
            let (column_index, index) = cursor_position as (usize, usize);
            for entity in &mut self.columns[column_index].entities {
                let (start, end) = entity.indices();
                match *entity {
                    CursorEntity::Word { .. } => {
                        if start <= index && index < end {
                            return Some(entity);
                        }
                    }
                    CursorEntity::Brackets { .. } => {
                        if start == index {
                            return Some(entity);
                        }
                    }
                }
            }
        }
        None
    }

    pub fn new(difficulty: i32) -> GameState {
        // Generate the (cosmetic) addresses along the left and right. We'll generate them between
        // F000 and F900 to get some "hexy" addresses.
        let mut rng = rand::thread_rng();
        let starting_address = rng.gen_range(0xF000, 0xF900);
        let mut addresses = (starting_address..).step(0xC);

        let word_length = difficulty as usize;
        let num_words = 12;
        let words = GameState::generate_words(num_words, word_length);

        let left_column = Column::new(addresses.by_ref().take(ROWS as usize).collect(),
                                      &words[..words.len() / 2]);
        let right_column = Column::new(addresses.take(ROWS as usize).collect(),
                                       &words[words.len() / 2..]);

        let mut words = left_column.words();
        words.extend(right_column.words());
        let correct_word = rand::sample(&mut rng, words.iter(), 1).first().unwrap().clone();

        GameState {
            attempts: 4,
            columns: [left_column, right_column],
            cursor_position: (0, 0),
            correct_word: correct_word.clone(),
            is_playing: true,
            entries: vec![],
            status: None,
        }
    }

    fn generate_words(num_words: i32, length: usize) -> Vec<String> {
        let dict = File::open("/usr/share/dict/words").unwrap();
        let words = BufReader::new(dict)
            .lines()
            .map(|word| word.unwrap())
            .filter(|word| word.chars().count() == length)
            .filter(|word| word.is_ascii())
            .filter(|word| word.chars().next().unwrap().is_lowercase())
            .filter(|word| word.chars().all(|c: char| c.is_alphabetic()));

        let mut rng = rand::thread_rng();
        rand::sample(&mut rng, words, num_words as usize)
            .iter()
            .map(|s| s.clone())
            .collect()
    }


    fn select_entity(&mut self) {
        if let Some(entity) = self.get_entity_at_cursor().cloned() {
            match entity {
                CursorEntity::Word { word, .. } => self.guess_word(&word),
                CursorEntity::Brackets { .. } => self.trigger_brackets(),
            }
        }
    }

    fn remove_dud(&mut self) {
        let mut entities: Vec<&mut CursorEntity> = vec![];
        for (i, column) in &mut self.columns.iter_mut().enumerate() {
            entities.extend(column.entities.get_mut(i));
        }

        for entity in entities {
            match *entity {
                CursorEntity::Word { ref word, ref mut removed, .. } => {
                    if *word != self.correct_word && !*removed {
                        *removed = true;
                        return;
                    }
                }
                _ => continue,
            }
        }
    }

    fn trigger_brackets(&mut self) {
        match *self.get_entity_at_cursor_mut().unwrap() {
            CursorEntity::Brackets { ref mut consumed, .. } => {
                *consumed = true;
            }
            _ => panic!("expected brackets to be under cursor"),
        }

        let mut rng = rand::thread_rng();
        let replenish_allowance = rng.gen_weighted_bool(3);
        if replenish_allowance {
            self.entries.push(Entry::AllowanceReplenish);
            self.attempts = STARTING_ATTEMPTS;
        } else {
            self.entries.push(Entry::DudRemoval);
            self.remove_dud();
        }
    }

    fn guess_word(&mut self, word: &str) {
        self.attempts -= 1;
        match *self.get_entity_at_cursor_mut().unwrap() {
            CursorEntity::Word { ref mut guessed, .. } => {
                *guessed = true;
            }
            _ => panic!("expected word to be under cursor"),
        }

        if word == self.correct_word {
            self.entries.push(Entry::Correct { word: word.to_string() });
            self.status = Some(GameEnding::Won);
        } else {
            let num_correct = word.chars()
                .enumerate()
                .filter(|&(i, c)| c == self.correct_word.chars().nth(i).unwrap())
                .count();
            self.entries.push(Entry::Incorrect {
                word: word.to_string(),
                num_correct: num_correct as i32,
            });
            if self.attempts == 0 {
                self.status = Some(GameEnding::Lost);
            }
        }
    }
}

pub struct Column {
    pub addresses: Vec<u16>,
    word_data: [char; CHARACTERS_PER_COLUMN as usize],
    entities: Vec<CursorEntity>,
}

impl Column {
    fn words(&self) -> Vec<String> {
        self.entities
            .iter()
            .filter_map(|e| {
                let e = e.clone();
                match e {
                    CursorEntity::Word { word, .. } => Some(word),
                    _ => None,
                }
            })
            .collect()
    }

    pub fn render_word_data(&self) -> String {
        let mut data = self.word_data.to_vec().into_iter().collect::<Vec<char>>();

        for entity in &self.entities {
            match *entity {
                CursorEntity::Word { ref word, guessed, index, removed, .. } => {
                    for (char_index, character) in word.to_ascii_uppercase()
                        .chars()
                        .enumerate() {
                        let char_position: usize = index + char_index;
                        data[char_position] = if !guessed && !removed {
                            character
                        } else {
                            '.'
                        };
                    }
                }
                CursorEntity::Brackets { pair, indices, consumed, .. } => {
                    let (l_index, r_index) = indices;
                    if !consumed {
                        let (l_bracket, r_bracket) = pair;
                        data[l_index] = l_bracket;
                        data[r_index] = r_bracket;
                    } else {
                        for index in (l_index..r_index) {
                            data[index] = '.';
                        }
                    }
                }
            }
        }

        data.into_iter().collect::<String>()
    }

    fn new(addresses: Vec<u16>, words: &[String]) -> Column {
        let word_length = words.iter().next().unwrap().len();
        let word_entities = words.iter()
            .enumerate()
            .map(|(index, word)| {
                let mut rng = rand::thread_rng();
                let chars_available = CHARACTERS_PER_COLUMN as usize / words.len();
                let offset: usize = rng.gen_range(0, chars_available - word_length);
                CursorEntity::Word {
                    word: word.to_string(),
                    guessed: false,
                    index: index * chars_available + offset,
                    removed: false,
                }
            })
            .collect::<Vec<CursorEntity>>();

        let brackets = Column::generate_brackets(8, &word_entities);
        let mut entities = vec![];

        entities.extend(brackets);
        entities.extend(word_entities);

        let garbage_characters = Column::generate_characters();

        Column {
            addresses: addresses,
            word_data: garbage_characters,
            entities: entities,
        }
    }

    fn generate_characters() -> [char; CHARACTERS_PER_COLUMN as usize] {
        const GARBAGE_CHARACTERS: &'static str = r",|\!@#$%^&*-_+=.:;?,/";

        const NUM_CHARS: usize = CHARACTERS_PER_COLUMN as usize;
        let mut characters = ['\0'; NUM_CHARS];
        let mut rng = rand::thread_rng();

        let range = Range::new(0, GARBAGE_CHARACTERS.len());
        for character in characters.iter_mut() {
            let index = range.ind_sample(&mut rng);
            *character = GARBAGE_CHARACTERS.chars().nth(index).unwrap();
        }
        characters
    }

    fn generate_brackets(num_brackets: i32, words: &[CursorEntity]) -> Vec<CursorEntity> {
        const PAIRS: [(char, char); 4] = [('<', '>'), ('[', ']'), ('{', '}'), ('(', ')')];

        let bracket_length = 8;
        let valid_indices = (0..CHARACTERS_PER_COLUMN as usize).filter(|&i| {
            words.into_iter()
                .all(|word| {
                    let (start, end) = word.indices();
                    if i < start {
                        i + bracket_length < start
                    } else {
                        i > end && i + bracket_length < CHARACTERS_PER_COLUMN as usize
                    }
                })
        });
        let mut rng = rand::thread_rng();
        let range = Range::new(0, PAIRS.len());
        rand::sample(&mut rng, valid_indices, num_brackets as usize)
            .iter()
            .map(|&index| {
                CursorEntity::Brackets {
                    pair: PAIRS[range.ind_sample(&mut rng)],
                    indices: (index, index + bracket_length),
                    consumed: false,
                }
            })
            .collect::<Vec<CursorEntity>>()
    }
}
