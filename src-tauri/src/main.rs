// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod tools;
mod utils2;

fn main() {
    tools::init_db_if_needed().unwrap();
    tbox_lib::run()
}
