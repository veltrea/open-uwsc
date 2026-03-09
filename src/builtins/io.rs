use super::{Builtins, BuiltinModule, BuiltinFunc, RuntimeValue};
use std::collections::HashMap;
use std::process::Command;

pub struct IoBuiltins;

impl BuiltinModule for IoBuiltins {
    fn register(&self, r: &mut HashMap<String, BuiltinFunc>) {
        r.insert("EXEC".into(), exec_func);
        r.insert("DOSCMD".into(), doscmd);
        r.insert("POWERSHELL".into(), powershell);
        r.insert("FOPEN".into(), fopen);
        r.insert("FGET".into(), fget);
        r.insert("FPUT".into(), fput);
        r.insert("FCLOSE".into(), fclose);
        r.insert("STDOUT".into(), stdout);
        r.insert("STDERR".into(), stderr);
        r.insert("STDIN".into(), stdin);
        r.insert("GETDIR".into(), getdir);
        r.insert("DELETEFILE".into(), delete_file);
        r.insert("READINI".into(), read_ini);
        r.insert("WRITEINI".into(), write_ini);
        r.insert("DELETEINI".into(), delete_ini);
    }
}

fn exec_func(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("EXEC requires a command string".into()); }
    let cmd = args[0].to_string();
    match Command::new("cmd").args(&["/C", &cmd]).spawn() {
        Ok(child) => Ok(RuntimeValue::Integer(child.id() as i64)),
        Err(e) => Err(format!("EXEC failed: {}", e)),
    }
}

fn doscmd(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("DOSCMD requires a command string".into()); }
    let cmd = args[0].to_string();
    match Command::new("cmd").args(&["/C", &cmd]).output() {
        Ok(output) => Ok(RuntimeValue::String(String::from_utf8_lossy(&output.stdout).to_string())),
        Err(e) => Err(format!("DOSCMD failed: {}", e)),
    }
}

fn powershell(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("POWERSHELL requires a command string".into()); }
    let cmd = args[0].to_string();
    match Command::new("powershell").args(&["-Command", &cmd]).output() {
        Ok(output) => Ok(RuntimeValue::String(String::from_utf8_lossy(&output.stdout).to_string())),
        Err(e) => Err(format!("POWERSHELL failed: {}", e)),
    }
}

fn stdout(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("STDOUT requires an argument".into()); }
    let msg = args[0].to_string();
    println!("{}", msg);
    Ok(RuntimeValue::Empty)
}

fn stderr(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("STDERR requires an argument".into()); }
    let msg = args[0].to_string();
    eprintln!("{}", msg);
    Ok(RuntimeValue::Empty)
}

fn stdin(_b: &mut Builtins, _args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    use std::io::{self, Write};
    let mut input = String::new();
    io::stdout().flush().map_err(|e| e.to_string())?;
    io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
    Ok(RuntimeValue::String(input.trim_end_matches(['\r', '\n']).to_string()))
}

fn fopen(b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("FOPEN requires a file path".into()); }
    let path_str = args[0].to_string();
    let path = std::path::Path::new(&path_str);
    
    let mut lines = Vec::new();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(path) {
            lines = content.lines().map(|s| s.to_string()).collect();
        }
    }

    let id = (b.file_handles.len() + 1) as i64;
    b.file_handles.insert(id, crate::builtins::FileHandle {
        path: path_str,
        lines,
        modified: false,
    });

    Ok(RuntimeValue::Integer(id))
}

fn fget(b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("FGET requires a FileID".into()); }
    let id = args[0].as_i64();
    let line_idx = if args.len() > 1 { args[1].as_i64() as usize } else { 1 };
    let col_idx = if args.len() > 2 { Some(args[2].as_i64() as usize) } else { None };

    if let Some(handle) = b.file_handles.get(&id) {
        if line_idx > 0 && line_idx <= handle.lines.len() {
            let line = &handle.lines[line_idx - 1];
            if let Some(c) = col_idx {
                // Simplified column access: split by comma? UWSC FGET/FPUT often used with F_CSV
                // For now, let's just split by comma if col requested
                let parts: Vec<&str> = line.split(',').collect();
                if c > 0 && c <= parts.len() {
                    return Ok(RuntimeValue::String(parts[c-1].to_string()));
                }
                return Ok(RuntimeValue::String("".into()));
            }
            return Ok(RuntimeValue::String(line.clone()));
        }
        Ok(RuntimeValue::String("".into()))
    } else {
        Err(format!("Invalid FileID: {}", id))
    }
}

fn fput(b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.len() < 2 { return Err("FPUT requires FileID and value".into()); }
    let id = args[0].as_i64();
    let val = args[1].to_string();
    let line_idx = if args.len() > 2 { args[2].as_i64() as usize } else { 0 };
    let col_idx = if args.len() > 3 { Some(args[3].as_i64() as usize) } else { None };

    if let Some(handle) = b.file_handles.get_mut(&id) {
        handle.modified = true;
        if line_idx == 0 {
            // Append
            handle.lines.push(val);
        } else {
            // Ensure lines exist up to line_idx
            while handle.lines.len() < line_idx {
                handle.lines.push("".into());
            }
            if let Some(c) = col_idx {
                let mut parts: Vec<String> = handle.lines[line_idx - 1].split(',').map(|s| s.to_string()).collect();
                while parts.len() < c {
                    parts.push("".into());
                }
                parts[c-1] = val;
                handle.lines[line_idx - 1] = parts.join(",");
            } else {
                handle.lines[line_idx - 1] = val;
            }
        }
        Ok(RuntimeValue::Bool(true))
    } else {
        Err(format!("Invalid FileID: {}", id))
    }
}

fn fclose(b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("FCLOSE requires a FileID".into()); }
    let id = args[0].as_i64();
    if let Some(handle) = b.file_handles.remove(&id) {
        if handle.modified {
            if let Err(e) = std::fs::write(&handle.path, handle.lines.join("\n")) {
                return Err(format!("FCLOSE failed to write file: {}", e));
            }
        }
    }
    Ok(RuntimeValue::Bool(true))
}

fn getdir(b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("GETDIR requires a directory path".into()); }
    let dir_path = args[0].to_string();
    // Simplified: just list all files in directory for now.
    // In future, support wildcards (arg[1])
    let path = std::path::Path::new(&dir_path);
    if !path.is_dir() {
        return Ok(RuntimeValue::Integer(0));
    }

    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                files.push(name);
            }
        }
    }
    
    let count = files.len() as i64;
    b.getdir_files = files;
    Ok(RuntimeValue::Integer(count))
}

fn delete_file(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.is_empty() { return Err("DELETEFILE requires a file path".into()); }
    let path = args[0].to_string();
    // Simple deletion (no wildcard support yet)
    match std::fs::remove_file(path) {
        Ok(_) => Ok(RuntimeValue::Bool(true)),
        Err(_) => Ok(RuntimeValue::Bool(false)),
    }
}

fn read_ini(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.len() < 3 { return Err("READINI requires 3 arguments (Section, Key, File)".into()); }
    let section = args[0].to_string().to_lowercase();
    let key = args[1].to_string().to_lowercase();
    let file_path = args[2].to_string();

    if let Ok(content) = std::fs::read_to_string(&file_path) {
        let mut current_section = String::new();
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('[') && line.ends_with(']') {
                current_section = line[1..line.len()-1].to_lowercase();
            } else if current_section == section {
                if let Some(pos) = line.find('=') {
                    let k = line[..pos].trim().to_lowercase();
                    if k == key {
                        return Ok(RuntimeValue::String(line[pos+1..].trim().to_string()));
                    }
                }
            }
        }
    }
    Ok(RuntimeValue::String("".into()))
}

fn write_ini(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.len() < 4 { return Err("WRITEINI requires 4 arguments (Section, Key, Value, File)".into()); }
    let section = args[0].to_string();
    let key = args[1].to_string();
    let value = args[2].to_string();
    let file_path = args[3].to_string();

    let mut lines = Vec::new();
    if let Ok(content) = std::fs::read_to_string(&file_path) {
        lines = content.lines().map(|s| s.to_string()).collect();
    }

    let mut section_found = false;
    let mut key_found = false;
    let mut current_section = String::new();
    let mut new_lines = Vec::new();

    let target_section_lower = section.to_lowercase();
    let target_key_lower = key.to_lowercase();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if section_found && !key_found {
                // Key wasn't in the section, add it before next section
                new_lines.push(format!("{}={}", key, value));
                key_found = true;
            }
            current_section = trimmed[1..trimmed.len()-1].to_lowercase();
            if current_section == target_section_lower {
                section_found = true;
            }
        } else if current_section == target_section_lower {
            if let Some(pos) = trimmed.find('=') {
                let k = trimmed[..pos].trim().to_lowercase();
                if k == target_key_lower {
                    new_lines.push(format!("{}={}", key, value));
                    key_found = true;
                    continue;
                }
            }
        }
        new_lines.push(line);
    }

    if !section_found {
        new_lines.push(format!("[{}]", section));
        new_lines.push(format!("{}={}", key, value));
    } else if !key_found {
        new_lines.push(format!("{}={}", key, value));
    }

    if let Err(e) = std::fs::write(&file_path, new_lines.join("\n")) {
        return Err(format!("WRITEINI failed: {}", e));
    }
    Ok(RuntimeValue::Bool(true))
}

fn delete_ini(_b: &mut Builtins, args: &mut [RuntimeValue]) -> Result<RuntimeValue, String> {
    if args.len() < 3 { return Err("DELETEINI requires at least 3 arguments (Section, Key, File)".into()); }
    let section = args[0].to_string().to_lowercase();
    let key = args[1].to_string().to_lowercase();
    let file_path = args[2].to_string();

    let mut lines = Vec::new();
    if let Ok(content) = std::fs::read_to_string(&file_path) {
        lines = content.lines().map(|s| s.to_string()).collect();
    } else {
        return Ok(RuntimeValue::Bool(false));
    }

    let mut current_section = String::new();
    let mut new_lines = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed[1..trimmed.len()-1].to_lowercase();
        } else if current_section == section {
            if let Some(pos) = trimmed.find('=') {
                let k = trimmed[..pos].trim().to_lowercase();
                if k == key {
                    continue; // Skip this line to delete
                }
            }
        }
        new_lines.push(line);
    }

    if let Err(e) = std::fs::write(&file_path, new_lines.join("\n")) {
        return Err(format!("DELETEINI failed: {}", e));
    }
    Ok(RuntimeValue::Bool(true))
}
