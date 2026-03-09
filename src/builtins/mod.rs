use crate::value::RuntimeValue;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

pub mod array;
pub mod window;
pub mod input;
pub mod string;
pub mod string_ext;
pub mod json;
pub mod http;
pub mod math;
pub mod io;
pub mod image;
pub mod system;
pub mod stable;
pub mod experimental;
pub mod stubs;
pub mod inspection;
pub mod browser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub base_url: String,
    pub api_key: String,
    pub default_model: String,
    pub system_prompt: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:1234/v1".into(),
            api_key: "".into(),
            default_model: "local-model".into(),
            system_prompt: "You are a helpful assistant.".into(),
        }
    }
}

pub enum GuiRequest {
    Prompt { title: String, fields: Vec<String>, resp_tx: mpsc::Sender<Vec<String>> },
    Notify { message: String, duration_secs: f64 },
}

pub type BuiltinFunc = fn(&mut Builtins, &mut [RuntimeValue]) -> Result<RuntimeValue, String>;

pub trait BuiltinModule {
    fn register(&self, registry: &mut HashMap<String, BuiltinFunc>);
}

#[derive(Debug, Clone)]
pub struct DllFunction {
    pub name: String,
    pub params: Vec<crate::parser::Parameter>,
    pub ret_type: String,
    pub dll_path: String,
}

#[derive(Debug, Clone)]
pub struct FileHandle {
    pub path: String,
    pub lines: Vec<String>,
    pub modified: bool,
}

pub struct Builtins {
    pub(crate) gui_tx: Option<mpsc::Sender<GuiRequest>>,
    pub(crate) ai: AiConfig,
    pub(crate) registry: HashMap<String, BuiltinFunc>,
    pub last_img_x: i32,
    pub last_img_y: i32,
    pub last_img_id: i32,
    pub last_token_remaining: Option<String>,
    pub last_modified_array: Option<Vec<RuntimeValue>>,
    pub getdir_files: Vec<String>,
    pub file_handles: HashMap<i64, FileHandle>,
    pub g_time_yy: i32,
    pub g_time_mm: i32,
    pub g_time_dd: i32,
    pub g_time_hh: i32,
    pub g_time_nn: i32,
    pub g_time_ss: i32,
    pub g_time_zz: i32,
    pub g_time_ww: i32,
    pub dll_registry: HashMap<String, DllFunction>,
    pub loaded_libs: HashMap<String, Arc<libloading::Library>>,
    pub browser_tab: Option<Arc<headless_chrome::Tab>>,
    pub browser: Option<headless_chrome::Browser>,
}

impl Builtins {
    pub fn new(gui_tx: Option<mpsc::Sender<GuiRequest>>, ai: AiConfig) -> Self {
        let mut b = Self {
            gui_tx,
            ai,
            registry: HashMap::new(),
            last_img_x: -1,
            last_img_y: -1,
            last_img_id: 0,
            last_token_remaining: None,
            last_modified_array: None,
            getdir_files: Vec::new(),
            file_handles: HashMap::new(),
            g_time_yy: 0,
            g_time_mm: 0,
            g_time_dd: 0,
            g_time_hh: 0,
            g_time_nn: 0,
            g_time_ss: 0,
            g_time_zz: 0,
            g_time_ww: 0,
            dll_registry: HashMap::new(),
            loaded_libs: HashMap::new(),
            browser_tab: None,
            browser: None,
        };
        b.register_all();
        b
    }

    fn register_all(&mut self) {
        // Trait-based registration will go here
        window::WindowBuiltins.register(&mut self.registry);
        input::InputBuiltins.register(&mut self.registry);
        string::StringBuiltins.register(&mut self.registry);
        string_ext::StringExtBuiltins.register(&mut self.registry);
        inspection::InspectionBuiltins.register(&mut self.registry);
        browser::BrowserBuiltins.register(&mut self.registry);
        json::JsonBuiltins.register(&mut self.registry);
        http::HttpBuiltins.register(&mut self.registry);
        math::MathBuiltins.register(&mut self.registry);
        io::IoBuiltins.register(&mut self.registry);
        image::ImageBuiltins.register(&mut self.registry);
        system::SystemBuiltins.register(&mut self.registry);
        stable::StableBuiltins.register(&mut self.registry);
        experimental::ExperimentalBuiltins.register(&mut self.registry);
        stubs::StubsBuiltins.register(&mut self.registry);
        array::ArrayBuiltins.register(&mut self.registry);
    }

    pub fn call(&mut self, name: &str, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
        let name_up = name.to_uppercase();
        if let Some(func) = self.registry.get(&name_up) {
            func(self, args)
        } else if let Some(dll_func) = self.dll_registry.get(&name_up).cloned() {
            self.call_dll(&dll_func, args)
        } else {
            Err(format!("Builtin function not found: {}", name))
        }
    }

    fn load_lib(&mut self, path: &str) -> Result<Arc<libloading::Library>, String> {
        if let Some(lib) = self.loaded_libs.get(path) {
            return Ok(lib.clone());
        }
        let lib = unsafe {
            libloading::Library::new(path)
                .map_err(|e| format!("Failed to load DLL ({}): {}", path, e))?
        };
        let arclib = Arc::new(lib);
        self.loaded_libs.insert(path.to_string(), arclib.clone());
        Ok(arclib)
    }

    fn call_dll(&mut self, dll: &DllFunction, args: &[RuntimeValue]) -> Result<RuntimeValue, String> {
        let lib = self.load_lib(&dll.dll_path)?;
        unsafe {
            match args.len() {
                0 => {
                    match dll.ret_type.to_lowercase().as_str() {
                        "dword" | "uint" => {
                            let func: libloading::Symbol<unsafe extern "system" fn() -> u32> = lib.get(dll.name.as_bytes())
                                .map_err(|e| format!("Symbol not found: {}", e))?;
                            Ok(RuntimeValue::Integer(func() as i64))
                        }
                        "int" | "long" => {
                            let func: libloading::Symbol<unsafe extern "system" fn() -> i32> = lib.get(dll.name.as_bytes())
                                .map_err(|e| format!("Symbol not found: {}", e))?;
                            Ok(RuntimeValue::Integer(func() as i64))
                        }
                        _ => {
                            let func: libloading::Symbol<unsafe extern "system" fn()> = lib.get(dll.name.as_bytes())
                                .map_err(|e| format!("Symbol not found: {}", e))?;
                            func();
                            Ok(RuntimeValue::Empty)
                        }
                    }
                }
                _ => Err(format!("DLL call with {} arguments not yet implemented", args.len())),
            }
        }
    }
}
