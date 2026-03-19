#![allow(unused_imports)]
use crate::router::handlers::clean::{CleanHandler, CleanInput};
use crate::router::handlers::common::{
    sanitize_control_chars, strip_ansi_codes, CommandContext, CommandError, CommandResult,
    CommandStats,
};
use crate::router::handlers::html2md::{Html2mdHandler, Html2mdInput};
use crate::router::handlers::isclean::{IsCleanHandler, IsCleanInput};
use crate::router::handlers::parse::ParseHandler;
use crate::router::handlers::replace::{ReplaceHandler, ReplaceInput};
use crate::router::handlers::run::{RunHandler, RunInput};
use crate::router::handlers::search::{SearchHandler, SearchInput};
use crate::router::handlers::tail::{TailHandler, TailInput};
use crate::router::handlers::trim::{TrimHandler, TrimInput};
use crate::router::handlers::txt2md::{Txt2mdHandler, Txt2mdInput};
use crate::router::handlers::types::*;
use crate::router::Router;
use crate::{Commands, OutputFormat, ParseCommands};

mod command_stats;
mod handlers;
mod html2md_advanced;
mod html2md_basic;
mod malformed_input;
mod parse_bun;
mod parse_grep;
mod parse_jest;
mod parse_logs;
mod parse_logs_levels;
mod parse_npm;
mod parse_pnpm;
mod parse_pytest;
mod txt2md;
