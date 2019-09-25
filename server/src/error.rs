
pub use failure::format_err as err;

pub type Error = failure::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

