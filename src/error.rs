
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Generic {0}")]
    Generic(String),

    #[error("sqlx error occurred: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Can not parse variable: {input}")]
    Var {
        input: &'static str,
        #[source]
        source: std::env::VarError,
    },

    #[error(transparent)]
    AddParse(#[from] std::net::AddrParseError),
}