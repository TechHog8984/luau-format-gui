#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::{env, fs, thread};

use eframe::egui;
use egui_code_editor::{CodeEditor, ColorTheme, Syntax};

#[tokio::main]
async fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    let home = env::var("HOME").expect("failed to get HOME environment variable");
    let mut binary_path = "luau-format".to_string();

    let cmd = Command::new("luau-format").output();
    if cmd.is_err() {
        println!("downloading luau-format...");
        let url = if cfg!(windows) {
            "https://github.com/TechHog8984/luau-format/releases/latest/download/luau-format.exe"
                .to_string()
        } else {
            format!(
                "https://github.com/TechHog8984/luau-format/releases/latest/download/luau-format-{}",
                env::consts::ARCH
            ).to_string()
        };
        let mut path = format!("{}/.luau-format-gui", home);
        if !fs::exists(path.clone()).expect(format!("failed to stat {}", path).as_str()) {
            fs::create_dir(path.clone()).expect(format!("failed to create dir {}", path).as_str());
        }

        path += "/luau-format";
        let body = reqwest::get(url.clone())
            .await
            .expect("failed to make request to github releases")
            .bytes()
            .await
            .expect("failed to get body from github releases request");
        if body.eq("Not Found".as_bytes()) {
            panic!(
                "invalid file at url {}! is your architecture supported?",
                url
            );
        }
        fs::write(path.clone(), body)
            .expect(format!("failed to write to {}", path.clone()).as_str());
        println!("successfully downloaded luau-format from latest github release.");
        binary_path = path;
    }
    eframe::run_native(
        "luau-format",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::new(binary_path)))),
    )
}

struct MyApp {
    binary_path: String,
    error: Arc<Mutex<Option<String>>>,

    input_file: Arc<Mutex<Option<PathBuf>>>,
    is_opening_input: bool,
    is_input_done: Arc<Mutex<bool>>,

    is_opening_output: Arc<Mutex<bool>>,

    formatted_code: String,
    editor_code: String,

    no_simplify: bool,
    minify: bool,
    lua_calls: bool,

    solve_record_table: bool,
    solve_list_table: bool,
}

impl MyApp {
    fn new(binary_path: String) -> Self {
        MyApp {
            binary_path,
            error: Arc::new(Mutex::new(None)),
            input_file: Arc::new(Mutex::new(None)),
            is_opening_input: false,
            is_input_done: Arc::new(Mutex::new(false)),
            is_opening_output: Arc::new(Mutex::new(false)),
            formatted_code: String::new(),
            editor_code: String::new(),
            no_simplify: false,
            minify: false,
            lua_calls: false,
            solve_record_table: false,
            solve_list_table: false,
        }
    }
}

impl MyApp {
    fn run_binary(&mut self) {
        if let Some(path) = self.input_file.lock().unwrap().as_ref() {
            let mut cmd = Command::new(self.binary_path.clone());
            cmd.arg(path);

            if self.no_simplify {
                cmd.arg("--nosimplify");
            }
            if self.minify {
                cmd.arg("--minify");
            }
            if self.lua_calls {
                cmd.arg("--lua_calls");
            }
            if self.solve_record_table {
                cmd.arg("--solve_record_table");
            }
            if self.solve_list_table {
                cmd.arg("--solve_list_table");
            }

            match cmd.output() {
                Ok(output) => {
                    if output.status.success() {
                        self.formatted_code = String::from_utf8_lossy(&output.stdout).to_string();
                        self.editor_code = self.formatted_code.clone();
                    } else {
                        self.error
                            .lock()
                            .unwrap()
                            .replace(String::from_utf8_lossy(&output.stderr).to_string());
                    }
                }
                Err(err) => {
                    self.error.lock().unwrap().replace(err.to_string());
                }
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("luau-format by techhog");

            ui.horizontal(|ui| {
                if ui.button("Open file...").clicked() && !self.is_opening_input {
                    *self.error.lock().unwrap() = None;
                    self.is_opening_input = true;
                    let input_file_arc = Arc::clone(&self.input_file);
                    let input_done_arc = Arc::clone(&self.is_input_done);

                    thread::spawn(move || {
                        let file = rfd::FileDialog::new().pick_file();
                        let mut input_file = input_file_arc.lock().unwrap();
                        *input_file = file;
                        let mut is_done = input_done_arc.lock().unwrap();
                        *is_done = true;
                    });
                }
                if ui.button("Save to file...").clicked()
                    && !*self.is_opening_output.lock().unwrap()
                {
                    *self.error.lock().unwrap() = None;
                    *self.is_opening_output.lock().unwrap() = true;

                    let opening_output_arc = Arc::clone(&self.is_opening_output);
                    let error_arc = Arc::clone(&self.error);

                    let code = self.editor_code.clone();
                    thread::spawn(move || {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_file_name("formatted.lua")
                            .save_file()
                        {
                            if let Err(err) = fs::write(path, code) {
                                error_arc.lock().unwrap().replace(err.to_string());
                            }
                        }
                        let mut opening_output = opening_output_arc.lock().unwrap();
                        *opening_output = false;
                    });
                }
            });

            ui.label("Options:");
            if ui
                .checkbox(&mut self.no_simplify, "no simplify - disable AstSimplifier")
                .clicked()
            {
                self.run_binary();
            };
            if ui
                .checkbox(&mut self.minify, "minify - minify code instead of beautify")
                .clicked()
            {
                self.run_binary();
            };
            if ui
                .checkbox(
                    &mut self.lua_calls,
                    "lua calls - solve lua calls such as math.max(1, 4)",
                )
                .clicked()
            {
                self.run_binary();
            };
            if ui
                .checkbox(
                    &mut self.solve_record_table,
                    "solve record table - solve Luraph's function table",
                )
                .clicked()
            {
                self.run_binary();
            };
            if ui
                .checkbox(
                    &mut self.solve_list_table,
                    "solve list table - solve Luraph's number table",
                )
                .clicked()
            {
                self.run_binary();
            };

            if ui.button("Reset editor...").clicked() {
                self.editor_code = self.formatted_code.clone();
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                CodeEditor::default()
                    .id_source("code editor")
                    .with_rows(1)
                    .with_fontsize(14.0)
                    .with_theme(ColorTheme::GRUVBOX)
                    .with_syntax(Syntax::lua())
                    .with_numlines(true)
                    .show(ui, &mut self.editor_code);
            });

            if *self.is_input_done.lock().unwrap() {
                *self.is_input_done.lock().unwrap() = false;
                self.is_opening_input = false;

                self.run_binary();
            }

            if let Some(err) = self.error.lock().unwrap().clone() {
                ui.label(format!("An error occured: {}", err));
            }
        });
    }
}
