#![allow(unused_imports)]
use crate::{OutputFormat, Commands, ParseCommands};
use crate::router::Router;
use crate::router::handlers::common::{CommandContext, CommandError, CommandResult, CommandStats,
    strip_ansi_codes, sanitize_control_chars};
use crate::router::handlers::types::*;
use crate::router::handlers::run::{RunHandler, RunInput};
use crate::router::handlers::search::{SearchHandler, SearchInput};
use crate::router::handlers::replace::{ReplaceHandler, ReplaceInput};
use crate::router::handlers::tail::{TailHandler, TailInput};
use crate::router::handlers::clean::{CleanHandler, CleanInput};
use crate::router::handlers::trim::{TrimHandler, TrimInput};
use crate::router::handlers::html2md::{Html2mdHandler, Html2mdInput};
use crate::router::handlers::txt2md::{Txt2mdHandler, Txt2mdInput};
use crate::router::handlers::isclean::{IsCleanHandler, IsCleanInput};
use crate::router::handlers::parse::ParseHandler;

mod malformed_input;
mod command_stats;
mod handlers;
mod html2md_basic;
mod html2md_advanced;
mod txt2md;
mod parse_pytest;
mod parse_jest;
mod parse_grep;
mod parse_npm;
mod parse_pnpm;
mod parse_bun;
mod parse_logs;
mod parse_logs_levels;
