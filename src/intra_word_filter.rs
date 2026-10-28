// A word-delimiter filter, which tantivy lacks: SplitCompoundWords only
// cuts words into dictionary entries.

use tantivy::tokenizer::{Token, TokenFilter, TokenStream, Tokenizer};

/// Splits each word into its parts, so that `data-base`, `DataBase` and
/// `database` are all found by `data` or `base`:
///
/// * Delimiters: `snake_case` gives `snake`, `case`; `_private_` gives
///   `private`.
/// * Case changes, with `split_words`: `DataBase` gives `Data`, `Base`.
/// * Letters next to digits, with `split_nums`: `utf8` gives `utf`, `8`.
/// * Possessives: `Alice's` gives `Alice`.
/// * Merged runs, with `merge_words` and `merge_nums`: `DataBase2-0` gives
///   `Data`, `Base`, `DataBase`, `2`, `0`, `20`.
///
/// Each part takes the next position and a merged run the position of its
/// last part: `DataBase XL` indexes `Data` 0, `Base` 1, `DataBase` 1, `XL`
/// 2. Parts keep their word's offsets. Put the filter before lowercasing
/// and before filters that drop tokens, as it renumbers what follows.
#[derive(Clone)]
pub struct IntraWordFilter {
    delims: String,
    split_words: bool,
    split_nums: bool,
    merge_words: bool,
    merge_nums: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Letters,
    Digits,
}

fn is_cased(c: char) -> bool {
    c.is_lowercase() || c.is_uppercase()
}

/// Cased numerals such as Roman `Ⅱ` count as letters, as in Python.
fn is_digit(c: char) -> bool {
    c.is_numeric() && !is_cased(c)
}

impl IntraWordFilter {
    /// The default delimiters.
    pub const DELIMS: &'static str = "-_'\"()!@#$%^&*[]{}<>\\|;:,./?`~=+";

    pub fn new(
        delims: &str,
        split_words: bool,
        split_nums: bool,
        merge_words: bool,
        merge_nums: bool,
    ) -> IntraWordFilter {
        IntraWordFilter {
            delims: delims.to_owned(),
            split_words,
            split_nums,
            merge_words,
            merge_nums,
        }
    }

    fn is_delim(&self, c: char) -> bool {
        self.delims.contains(c)
    }

    fn is_boundary(&self, prev: char, c: char) -> bool {
        if self.split_words && prev.is_lowercase() && c.is_uppercase() {
            return true;
        }
        if !self.split_nums {
            return false;
        }
        if is_digit(prev) {
            return is_cased(c);
        }
        is_digit(c) && is_cased(prev)
    }

    /// A word in one case, or of digits, stays whole.
    fn is_plain(word: &str) -> bool {
        let mut alpha = true;
        let mut digits = true;
        let mut lower = false;
        let mut upper = false;
        for c in word.chars() {
            alpha &= c.is_alphabetic();
            digits &= is_digit(c);
            lower |= c.is_lowercase();
            upper |= c.is_uppercase();
        }
        (alpha && lower != upper) || digits
    }

    fn kind(&self, part: &str) -> Option<Kind> {
        if self.merge_words && part.chars().all(char::is_alphabetic) {
            return Some(Kind::Letters);
        }
        if self.merge_nums && part.chars().all(is_digit) {
            return Some(Kind::Digits);
        }
        None
    }

    /// Fills `cuts` with the byte ranges of the word's subwords, in order,
    /// before any merging.
    fn split(&self, word: &str, cuts: &mut Vec<(usize, usize)>) {
        cuts.clear();
        let mut start = 0;
        if word.contains('\'') {
            let bytes = word.as_bytes();
            let mut i = 1;
            while i + 1 < bytes.len() {
                if self.is_possessive(word, i) {
                    self.runs(word, start, i, cuts);
                    start = i + 2;
                    i += 2;
                    continue;
                }
                i += 1;
            }
        }
        if start < word.len() {
            self.runs(word, start, word.len(), cuts);
        }
    }

    /// Whether a possessive 's starts at byte `i`: after a cased letter and
    /// before the end of the word or a delimiter.
    fn is_possessive(&self, word: &str, i: usize) -> bool {
        let bytes = word.as_bytes();
        bytes[i] == b'\''
            && (bytes[i + 1] == b's' || bytes[i + 1] == b'S')
            && word[..i].chars().next_back().is_some_and(is_cased)
            && word[i + 2..].chars().next().is_none_or(|c| self.is_delim(c))
    }

    /// Cuts each run between delimiters in `word[from..to]`.
    fn runs(
        &self,
        word: &str,
        from: usize,
        to: usize,
        cuts: &mut Vec<(usize, usize)>,
    ) {
        let mut start = None;
        for (i, c) in word[from..to].char_indices() {
            match (self.is_delim(c), start) {
                (true, Some(s)) => {
                    self.cut(word, s, from + i, cuts);
                    start = None;
                }
                (false, None) => start = Some(from + i),
                _ => {}
            }
        }
        if let Some(s) = start {
            self.cut(word, s, to, cuts);
        }
    }

    /// Cuts `word[from..to]` at case and letter-digit changes. The character
    /// after a change is not checked against the one that made it, so `a1B`
    /// gives `a` and `1B`.
    fn cut(
        &self,
        word: &str,
        from: usize,
        to: usize,
        cuts: &mut Vec<(usize, usize)>,
    ) {
        let mut start = from;
        if self.split_words || self.split_nums {
            let mut chars = word[from..to].char_indices();
            if let Some((_, mut prev)) = chars.next() {
                while let Some((i, c)) = chars.next() {
                    if !self.is_boundary(prev, c) {
                        prev = c;
                        continue;
                    }
                    cuts.push((start, from + i));
                    start = from + i;
                    match chars.next() {
                        Some((_, c)) => prev = c,
                        None => break,
                    }
                }
            }
        }
        cuts.push((start, to));
    }

    /// Pushes the word's subwords as tokens, with each merged run right
    /// after its parts, from `position` on. Returns the next free position.
    fn merge(
        &self,
        token: &Token,
        cuts: &[(usize, usize)],
        parts: &mut Vec<Token>,
        mut position: usize,
    ) -> usize {
        let mut run = String::new();
        let mut run_len = 0;
        let mut last = None;
        for &(from, to) in cuts {
            let part = &token.text[from..to];
            let kind = self.kind(part);
            if run_len == 0 || kind.is_none() || kind != last {
                if run_len > 1 {
                    parts.push(Token {
                        text: run.clone(),
                        position: position - 1,
                        ..*token
                    });
                }
                run.clear();
                run_len = 0;
                last = kind;
            }
            if kind.is_some() {
                run.push_str(part);
            }
            run_len += 1;
            parts.push(Token {
                text: part.to_owned(),
                position,
                ..*token
            });
            position += 1;
        }
        if run_len > 1 {
            parts.push(Token {
                text: run,
                position: position - 1,
                ..*token
            });
        }
        position
    }
}

impl TokenFilter for IntraWordFilter {
    type Tokenizer<T: Tokenizer> = IntraWordFilterWrapper<T>;

    fn transform<T: Tokenizer>(self, tokenizer: T) -> Self::Tokenizer<T> {
        IntraWordFilterWrapper {
            filter: self,
            inner: tokenizer,
            cuts: Vec::new(),
            parts: Vec::new(),
        }
    }
}

#[derive(Clone)]
pub struct IntraWordFilterWrapper<T> {
    filter: IntraWordFilter,
    inner: T,
    cuts: Vec<(usize, usize)>,
    parts: Vec<Token>,
}

impl<T: Tokenizer> Tokenizer for IntraWordFilterWrapper<T> {
    type TokenStream<'a> = IntraWordTokenStream<'a, T::TokenStream<'a>>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        self.cuts.clear();
        self.parts.clear();
        IntraWordTokenStream {
            filter: &self.filter,
            tail: self.inner.token_stream(text),
            cuts: &mut self.cuts,
            parts: &mut self.parts,
            position: None,
        }
    }
}

pub struct IntraWordTokenStream<'a, T> {
    filter: &'a IntraWordFilter,
    tail: T,
    cuts: &'a mut Vec<(usize, usize)>,
    // The current word's subwords in reverse, so that `parts.pop()` yields
    // them in order; empty while the current word stays whole.
    parts: &'a mut Vec<Token>,
    // The next position to hand out, taken from the first word.
    position: Option<usize>,
}

impl<T: TokenStream> IntraWordTokenStream<'_, T> {
    /// Renumbers the current word if it stays whole, or fills `parts` with
    /// its subwords. Returns false if it has none, being delimiters only.
    fn expand(&mut self) -> bool {
        let token = self.tail.token();
        let position = *self.position.get_or_insert(token.position);
        if !IntraWordFilter::is_plain(&token.text) {
            self.filter.split(&token.text, self.cuts);
            if self.cuts.as_slice() != [(0, token.text.len())] {
                let next =
                    self.filter.merge(token, self.cuts, self.parts, position);
                self.parts.reverse();
                self.position = Some(next);
                return !self.parts.is_empty();
            }
        }
        self.tail.token_mut().position = position;
        self.position = Some(position + 1);
        true
    }
}

impl<T: TokenStream> TokenStream for IntraWordTokenStream<'_, T> {
    fn advance(&mut self) -> bool {
        self.parts.pop();
        if !self.parts.is_empty() {
            return true;
        }
        loop {
            if !self.tail.advance() {
                return false;
            }
            if self.expand() {
                return true;
            }
        }
    }

    fn token(&self) -> &Token {
        self.parts.last().unwrap_or_else(|| self.tail.token())
    }

    fn token_mut(&mut self) -> &mut Token {
        self.parts
            .last_mut()
            .unwrap_or_else(|| self.tail.token_mut())
    }
}
