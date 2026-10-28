use tantivy::tokenizer::{Token, TokenFilter, TokenStream, Tokenizer};

/// Removes tokens shorter than a given number of characters, the twin of
/// tantivy's `RemoveLongFilter`. As with that filter, a removed token
/// leaves a gap in the positions.
#[derive(Clone)]
pub struct RemoveShortFilter {
    length_limit: usize,
}

impl RemoveShortFilter {
    /// Creates a `RemoveShortFilter` given a limit in characters.
    pub fn limit(length_limit: usize) -> RemoveShortFilter {
        RemoveShortFilter { length_limit }
    }
}

impl TokenFilter for RemoveShortFilter {
    type Tokenizer<T: Tokenizer> = RemoveShortFilterWrapper<T>;

    fn transform<T: Tokenizer>(self, tokenizer: T) -> Self::Tokenizer<T> {
        RemoveShortFilterWrapper {
            length_limit: self.length_limit,
            inner: tokenizer,
        }
    }
}

#[derive(Clone)]
pub struct RemoveShortFilterWrapper<T> {
    length_limit: usize,
    inner: T,
}

impl<T: Tokenizer> Tokenizer for RemoveShortFilterWrapper<T> {
    type TokenStream<'a> = RemoveShortFilterStream<T::TokenStream<'a>>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        RemoveShortFilterStream {
            length_limit: self.length_limit,
            tail: self.inner.token_stream(text),
        }
    }
}

pub struct RemoveShortFilterStream<T> {
    length_limit: usize,
    tail: T,
}

impl<T: TokenStream> TokenStream for RemoveShortFilterStream<T> {
    fn advance(&mut self) -> bool {
        while self.tail.advance() {
            let text = &self.tail.token().text;
            if text.chars().take(self.length_limit).count() == self.length_limit
            {
                return true;
            }
        }
        false
    }

    fn token(&self) -> &Token {
        self.tail.token()
    }

    fn token_mut(&mut self) -> &mut Token {
        self.tail.token_mut()
    }
}
