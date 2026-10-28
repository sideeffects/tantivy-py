use pyo3::{exceptions::PyValueError, prelude::*};
use tantivy::tokenizer as tvt;

use crate::{
    intra_word_filter::IntraWordFilter,
    remove_short_filter::RemoveShortFilter,
};

/// All Tantivy's built-in tokenizers in one place.
/// Each static method, e.g. Tokenizer.simple(),
/// creates a wrapper around a Tantivy tokenizer.
///
/// ## Example:
///
/// ```python
/// tokenizer = Tokenizer.regex(r"\w+")
/// ```
///
/// ## Usage
///
/// In general, tokenizer objects' only reason
/// for existing is to be passed to
/// TextAnalyzerBuilder(tokenizer=<tokenizer>)
///
/// https://docs.rs/tantivy/latest/tantivy/tokenizer/index.html
///
// ## Implementation details:
//
// This is a complex enum. Each variant is a struct
// that defines the arguments accepted by the
// corresponding tokenizer's constructor.
// The enum members, e.g. _Raw, are not instantiated
// directly because our version of pyo3 (0.21.0)
// does not have the #[pyo3(constructor = ...)],
// attribute yet, making it more sensible to
// define constructor signatures using a separate method.
#[pyclass(module = "tantivy.tokenizer")]
#[derive(Debug)]
pub enum Tokenizer {
    _Raw {},
    _Simple {},
    _Whitespace {},
    _Regex {
        pattern: String,
    },
    _Ngram {
        min_gram: usize,
        max_gram: usize,
        prefix_only: bool,
    },
    _Facet {},
}

#[pymethods]
impl Tokenizer {
    /// SimpleTokenizer
    #[staticmethod]
    fn simple() -> PyResult<Tokenizer> {
        Ok(Tokenizer::_Simple {})
    }

    /// WhitespaceTokenizer
    #[staticmethod]
    fn whitespace() -> PyResult<Tokenizer> {
        Ok(Tokenizer::_Whitespace {})
    }

    /// Raw Tokenizer
    #[staticmethod]
    fn raw() -> PyResult<Tokenizer> {
        Ok(Tokenizer::_Raw {})
    }

    /// FacetTokenizer
    #[staticmethod]
    fn facet() -> PyResult<Tokenizer> {
        Ok(Tokenizer::_Facet {})
    }

    /// Regextokenizer
    #[staticmethod]
    fn regex(pattern: String) -> PyResult<Tokenizer> {
        Ok(Tokenizer::_Regex { pattern })
    }

    /// NgramTokenizer
    ///
    /// Args:
    /// - min_gram (int): Minimum character length of each ngram.
    /// - max_gram (int): Maximum character length of each ngram.
    /// - prefix_only (bool, optional): If true, ngrams must count from the start of the word.
    #[pyo3(signature=(min_gram=2,max_gram=3,prefix_only=false))]
    #[staticmethod]
    fn ngram(
        min_gram: usize,
        max_gram: usize,
        prefix_only: bool,
    ) -> PyResult<Tokenizer> {
        Ok(Tokenizer::_Ngram {
            min_gram,
            max_gram,
            prefix_only,
        })
    }

    fn __repr__(&self) -> String {
        format!("tantivy.Tokenizer({:?})", &self)
    }
}

/// All Tantivy's builtin TokenFilters.
///
/// ## Example
///
/// ```python
/// filter = Filter.alpha_num()
/// ```
///
/// ## Usage
///
/// In general, filter objects exist to
/// be passed to the filter() method
/// of a TextAnalyzerBuilder instance.
///
/// https://docs.rs/tantivy/latest/tantivy/tokenizer/index.html
///
// ## Implementation details:
//
// This is a complex enum. Each variant is a struct
// that defines the arguments accepted by the
// corresponding tokenizer's constructor.
// The enum members, e.g. _AlphaNum, are not instantiated
// directly because our version of pyo3 (0.21.0)
// does not have the #[pyo3(constructor = ...)],
// attribute yet, making it more sensible to
// define constructor signatures using a separate method.
#[pyclass(module = "tantivy.tokenizer")]
#[derive(Debug)]
pub enum Filter {
    _AlphaNumOnly {},
    _AsciiFolding {},
    _LowerCaser {},
    _RemoveLong { length_limit: usize },
    _RemoveShort { length_limit: usize },
    _Stemmer { language: String },
    _StopWord { language: String },
    _CustomStopWord { stopwords: Vec<String> },
    _SplitCompound { constituent_words: Vec<String> },
    _IntraWord {
        delims: String,
        split_words: bool,
        split_nums: bool,
        merge_words: bool,
        merge_nums: bool,
    },
}

#[pymethods]
impl Filter {
    /// AlphaNumOnlyFilter
    #[staticmethod]
    fn alphanum_only() -> PyResult<Filter> {
        Ok(Filter::_AlphaNumOnly {})
    }

    /// AsciiFoldingFilter
    #[staticmethod]
    fn ascii_fold() -> PyResult<Filter> {
        Ok(Filter::_AsciiFolding {})
    }

    #[staticmethod]
    fn lowercase() -> PyResult<Filter> {
        Ok(Filter::_LowerCaser {})
    }

    /// RemoveLongFilter
    ///
    /// Args:
    /// - length_limit (int): max character length of token.
    #[staticmethod]
    fn remove_long(length_limit: usize) -> PyResult<Filter> {
        Ok(Filter::_RemoveLong { length_limit })
    }

    /// RemoveShortFilter
    ///
    /// Removes tokens shorter than the limit, as remove_long() removes
    /// the longer ones. A removed token leaves a gap in the positions.
    ///
    /// Args:
    /// - length_limit (int): min character length of token.
    #[staticmethod]
    fn remove_short(length_limit: usize) -> PyResult<Filter> {
        Ok(Filter::_RemoveShort { length_limit })
    }

    /// Stemmer
    #[staticmethod]
    fn stemmer(language: String) -> PyResult<Filter> {
        Ok(Filter::_Stemmer { language })
    }

    /// StopWordFilter (builtin stop word list)
    ///
    /// Args:
    /// - language (string): Stop words list language.
    ///   Valid values: {
    ///     "danish", "dutch", "english", "finnish", "french", "german", "hungarian",
    ///     "italian", "norwegian", "portuguese", "russian", "spanish", "swedish"
    ///   }
    ///
    /// Adding this filter to a builder raises ValueError for any other language,
    /// including stemmer languages without a builtin stop word list.
    // ## Implementation notes:
    // An enum would make more sense for `language`, but I'm not sure if it's worth it.
    #[staticmethod]
    fn stopword(language: String) -> PyResult<Filter> {
        Ok(Filter::_StopWord { language })
    }

    /// StopWordFilter (user-provided stop word list)
    ///
    /// This variant of Filter.stopword() lets you provide
    /// your own custom list of stopwords.
    ///
    /// Args:
    /// - stopwords (list(str)): a list of words to be removed.
    #[staticmethod]
    fn custom_stopword(stopwords: Vec<String>) -> PyResult<Filter> {
        Ok(Filter::_CustomStopWord { stopwords })
    }

    /// SplitCompoundWords
    ///
    /// https://docs.rs/tantivy/latest/tantivy/tokenizer/struct.SplitCompoundWords.html
    ///
    /// Args:
    /// - constituent_words (list(string)): words that make up compound word (must be in order).
    ///
    /// Example:
    ///
    /// ```python
    /// # useless, contrived example:
    /// compound_spliter = Filter.split_compounds(['butter', 'fly'])
    /// # Will split 'butterfly' -> ['butter', 'fly'],
    /// # but won't split 'buttering' or 'buttercupfly'
    /// ```
    #[staticmethod]
    fn split_compound(constituent_words: Vec<String>) -> PyResult<Filter> {
        Ok(Filter::_SplitCompound { constituent_words })
    }

    /// IntraWordFilter
    ///
    /// Splits each word into its parts: at delimiter characters, where a
    /// lower-case letter meets an upper-case one, between letters and
    /// digits, and dropping a possessive 's. A run of letter parts or of
    /// digit parts can also be added joined up, right after the run. Each
    /// part takes the next position and a merged run the position of its
    /// last part, and the words after it are renumbered, closing any gap a
    /// filter before it left. Put the filter before Filter.lowercase() and
    /// before filters that drop tokens.
    ///
    /// Args:
    /// - delims (str, optional): the characters that split a word.
    /// - split_words (bool, optional): split at case changes,
    ///   `DataBase` -> `Data`, `Base`.
    /// - split_nums (bool, optional): split between letters and digits,
    ///   `utf8` -> `utf`, `8`.
    /// - merge_words (bool, optional): add each run of letter parts joined
    ///   up, `snake_case` -> `snake`, `case`, `snakecase`.
    /// - merge_nums (bool, optional): add each run of digit parts joined
    ///   up, `42-7` -> `42`, `7`, `427`.
    #[pyo3(signature=(delims=IntraWordFilter::DELIMS, split_words=true, split_nums=true, merge_words=false, merge_nums=false))]
    #[staticmethod]
    fn intra_word(
        delims: &str,
        split_words: bool,
        split_nums: bool,
        merge_words: bool,
        merge_nums: bool,
    ) -> PyResult<Filter> {
        Ok(Filter::_IntraWord {
            delims: delims.to_owned(),
            split_words,
            split_nums,
            merge_words,
            merge_nums,
        })
    }

    fn __repr__(&self) -> String {
        format!("tantivy.Filter(kind={:?})", &self)
    }
}

fn parse_language(lang: &str) -> Result<tvt::Language, String> {
    match lang.to_lowercase().as_str() {
        "arabic" => Ok(tvt::Language::Arabic),
        "danish" => Ok(tvt::Language::Danish),
        "dutch" => Ok(tvt::Language::Dutch),
        "english" => Ok(tvt::Language::English),
        "finnish" => Ok(tvt::Language::Finnish),
        "french" => Ok(tvt::Language::French),
        "german" => Ok(tvt::Language::German),
        "greek" => Ok(tvt::Language::Greek),
        "hungarian" => Ok(tvt::Language::Hungarian),
        "italian" => Ok(tvt::Language::Italian),
        "norwegian" => Ok(tvt::Language::Norwegian),
        "portuguese" => Ok(tvt::Language::Portuguese),
        "romanian" => Ok(tvt::Language::Romanian),
        "russian" => Ok(tvt::Language::Russian),
        "spanish" => Ok(tvt::Language::Spanish),
        "swedish" => Ok(tvt::Language::Swedish),
        "tamil" => Ok(tvt::Language::Tamil),
        "turkish" => Ok(tvt::Language::Turkish),
        _ => Err(format!("Unsupported language: {}", lang)),
    }
}

/// Tantivy's TextAnalyzer
///
/// Do not instantiate this class directly.
/// Use the `TextAnalyzerBuilder` class instead.
#[derive(Clone)]
#[pyclass(module = "tantivy.tantivy")]
pub(crate) struct TextAnalyzer {
    pub(crate) analyzer: tvt::TextAnalyzer,
}

#[pymethods]
impl TextAnalyzer {
    /// Tokenize a string
    /// Args:
    /// - text (string): text to tokenize.
    /// Returns:
    /// - list(string): a list of tokens/words.
    fn analyze(&mut self, text: &str) -> Vec<String> {
        let mut token_stream = self.analyzer.token_stream(text);
        let mut tokens = Vec::new();

        while token_stream.advance() {
            tokens.push(token_stream.token().text.clone());
        }
        tokens
    }

    /// Tokenize a string, keeping each token's position
    /// Args:
    /// - text (string): text to tokenize.
    /// Returns:
    /// - list((int, string)): the position and text of each token, as
    ///   Query.phrase_query() takes them. A filter that drops a token
    ///   leaves a gap in the positions.
    fn analyze_with_positions(&mut self, text: &str) -> Vec<(usize, String)> {
        let mut token_stream = self.analyzer.token_stream(text);
        let mut tokens = Vec::new();

        while token_stream.advance() {
            let token = token_stream.token();
            tokens.push((token.position, token.text.clone()));
        }
        tokens
    }
}

/// Tantivy's TextAnalyzerBuilder
///
/// # Example
///
/// ```python
/// my_analyzer: TextAnalyzer = (
///     TextAnalyzerBuilder(Tokenizer.simple())
///     .filter(Filter.lowercase())
///     .filter(Filter.ngram())
///     .build()
/// )
/// ```
///
/// https://docs.rs/tantivy/latest/tantivy/tokenizer/struct.TextAnalyzerBuilder.html
#[pyclass]
pub struct TextAnalyzerBuilder {
    builder: Option<tvt::TextAnalyzerBuilder>,
}

#[pymethods]
impl TextAnalyzerBuilder {
    #[new]
    fn new(tokenizer: &Tokenizer) -> PyResult<Self> {
        let builder: tvt::TextAnalyzerBuilder = match tokenizer {
            Tokenizer::_Raw {} => {
                tvt::TextAnalyzer::builder(tvt::RawTokenizer::default())
                    .dynamic()
            }
            Tokenizer::_Simple {} => {
                tvt::TextAnalyzer::builder(tvt::SimpleTokenizer::default())
                    .dynamic()
            }
            Tokenizer::_Whitespace {} => {
                tvt::TextAnalyzer::builder(tvt::WhitespaceTokenizer::default())
                    .dynamic()
            }
            Tokenizer::_Regex { pattern } => tvt::TextAnalyzer::builder(
                tvt::RegexTokenizer::new(pattern).map_err(|e| {
                    PyErr::new::<PyValueError, _>(format!(
                        "Invalid regex pattern: {}",
                        e
                    ))
                })?, // tvt::RegexTokenizer::new(pattern) .unwrap(),
            )
            .dynamic(),
            Tokenizer::_Ngram {
                min_gram,
                max_gram,
                prefix_only,
            } => tvt::TextAnalyzer::builder(
                tvt::NgramTokenizer::new(*min_gram, *max_gram, *prefix_only)
                    .unwrap(),
            )
            .dynamic(),
            Tokenizer::_Facet {} => {
                tvt::TextAnalyzer::builder(tvt::FacetTokenizer::default())
                    .dynamic()
            }
        };

        Ok(TextAnalyzerBuilder {
            builder: Some(builder.dynamic()),
        })
    }

    /// Add filter to the builder.
    ///
    /// Args:
    /// - filter (Filter): a Filter object.
    /// Returns:
    /// - TextAnalyzerBuilder: A new instance of the builder
    ///
    /// Note: The builder is _not_ mutated in-place.
    fn filter(&mut self, filter: &Filter) -> PyResult<Self> {
        if let Some(builder) = self.builder.take() {
            let new_builder: tvt::TextAnalyzerBuilder = match filter {
                Filter::_AlphaNumOnly {} => {
                    builder.filter_dynamic(tvt::AlphaNumOnlyFilter {})
                }
                Filter::_AsciiFolding {} => {
                    builder.filter_dynamic(tvt::AsciiFoldingFilter)
                }
                Filter::_LowerCaser {} => {
                    builder.filter_dynamic(tvt::LowerCaser)
                }
                Filter::_RemoveLong { length_limit } => builder.filter_dynamic(
                    tvt::RemoveLongFilter::limit(*length_limit),
                ),
                Filter::_RemoveShort { length_limit } => builder
                    .filter_dynamic(RemoveShortFilter::limit(*length_limit)),
                Filter::_Stemmer { language } => {
                    match parse_language(language) {
                        Ok(lang) => {
                            builder.filter_dynamic(tvt::Stemmer::new(lang))
                        }
                        Err(e) => {
                            return Err(PyErr::new::<
                                pyo3::exceptions::PyValueError,
                                _,
                            >(e))
                        }
                    }
                }
                Filter::_StopWord { language } => {
                    let lang = parse_language(language)
                        .map_err(PyValueError::new_err)?;
                    // Not every stemmer language has a builtin stop word list
                    let stop_words = tvt::StopWordFilter::new(lang)
                        .ok_or_else(|| {
                            PyValueError::new_err(format!(
                                "No builtin stop word list for language: {}",
                                language
                            ))
                        })?;
                    builder.filter_dynamic(stop_words)
                }
                Filter::_CustomStopWord { stopwords } => builder
                    .filter_dynamic(tvt::StopWordFilter::remove(
                        stopwords.clone(),
                    )),
                Filter::_SplitCompound { constituent_words } => builder
                    .filter_dynamic(
                        tvt::SplitCompoundWords::from_dictionary(
                            constituent_words,
                        )
                        .unwrap(),
                    ),
                Filter::_IntraWord {
                    delims,
                    split_words,
                    split_nums,
                    merge_words,
                    merge_nums,
                } => builder.filter_dynamic(IntraWordFilter::new(
                    delims,
                    *split_words,
                    *split_nums,
                    *merge_words,
                    *merge_nums,
                )),
            };
            Ok(TextAnalyzerBuilder {
                builder: Some(new_builder),
            })
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Builder has already been consumed",
            ))
        }
    }

    /// Build final TextAnalyzer object.
    ///
    /// Returns:
    /// - TextAnalyzer with tokenizer and filters baked in.
    ///
    /// Tip: TextAnalyzer's `analyze(text) -> tokens` method lets you
    /// easily check if your analyzer is working as expected.
    fn build(&mut self) -> PyResult<TextAnalyzer> {
        if let Some(builder) = self.builder.take() {
            Ok(TextAnalyzer {
                analyzer: builder.build(),
            })
        } else {
            Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "Builder has already been consumed",
            ))
        }
    }
}
