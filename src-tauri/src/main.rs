//! 步语桌面应用的可执行入口。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    buyu_lib::run()
}
