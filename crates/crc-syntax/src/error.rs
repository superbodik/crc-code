pub type Result<T> = std::result::Result<T, SyntaxError>;

#[derive(Debug, thiserror::Error)]
pub enum SyntaxError {
    #[error("the {0} grammar does not match this build of tree-sitter")]
    Grammar(&'static str),

    #[error("the {language} highlight query is invalid")]
    Query {
        language: &'static str,
        #[source]
        source: tree_sitter::QueryError,
    },

    #[error("parsing {0} produced no tree")]
    Parse(&'static str),
}
