#[macro_use]
extern crate lazy_static;

pub use yansi::Color;
use yansi::Paint;

pub use levels::*;
pub use logs::*;

pub mod blocks;
mod constants;
mod levels;
mod logs;
mod utils;

/// Initialises the library trying to enable the Windows ASCII support.
pub fn init_logger() {
    if cfg!(windows) && !Paint::enable_windows_ascii() {
        Paint::disable();
    }
}

/// Whether the ANSI colors are supported in the executing terminal or not.
pub fn is_ansi_supported() -> bool {
    if let Some(v) = std::env::var_os("CLICOLOR_FORCE") {
        if v != "0" {
            return true;
        }
    }

    if let Some(v) = std::env::var_os("CLICOLOR") {
        if v != "0" {
            return true;
        }
    }

    false
}

// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------
// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ansi_supported() {
        std::env::set_var("CLICOLOR_FORCE", "1");
        std::env::set_var("CLICOLOR", "1");

        assert!(is_ansi_supported());
        std::env::set_var("CLICOLOR_FORCE", "1");
        std::env::set_var("CLICOLOR", "0");

        assert!(is_ansi_supported());

        std::env::set_var("CLICOLOR_FORCE", "0");
        std::env::set_var("CLICOLOR", "1");

        assert!(is_ansi_supported());

        std::env::set_var("CLICOLOR_FORCE", "0");
        std::env::set_var("CLICOLOR", "0");

        assert!(!is_ansi_supported());
    }
}
