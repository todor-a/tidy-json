pub mod sort;

use clap::ValueEnum;

#[derive(Debug, Clone, ValueEnum)]
pub enum SortOrder {
    #[clap(name = "asc", alias = "alphabetical-asc", alias = "a")]
    AlphabeticalAsc,
    #[clap(name = "desc", alias = "alphabetical-desc", alias = "d")]
    AlphabeticalDesc,
    #[clap(name = "rand", alias = "random", alias = "r")]
    Random,
    #[clap(name = "key-length-asc", alias = "len-asc", alias = "kla")]
    KeyLengthAsc,
    #[clap(name = "key-length-desc", alias = "len-desc", alias = "kld")]
    KeyLengthDesc,
}
