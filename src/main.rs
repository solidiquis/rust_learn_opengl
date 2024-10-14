#![allow(dead_code)]
use anyhow::{format_err, Result};
use std::process::ExitCode;

mod _1_getting_started;

mod glutils;

fn main() -> ExitCode {
    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<()> {
    let mut clargs = std::env::args();
    let lesson = clargs
        .nth(1)
        .expect("expected number argument")
        .parse::<usize>()?;

    match lesson {
        1 => _1_getting_started::_1_hello_window::run(),
        2 => _1_getting_started::_2_hello_triangle::run(),
        3 => _1_getting_started::_2_hello_square::run(),
        4 => _1_getting_started::_3_shaders::run(),
        5 => _1_getting_started::_3_shaders_triangle::run(),
        _ => Err(format_err!("no lesson of number {lesson}")),
    }
}
