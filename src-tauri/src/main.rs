// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Usamos el nombre del package definido en Cargo.toml pero con guiones bajos
    analizador_caracteristicas_pc::run();
}
