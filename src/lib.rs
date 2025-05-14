use std::ffi::CStr;
use std::os::raw::c_char;
use std::process::Command;

/// Converts a string literal into a C-compatible string pointer (`*const c_char`).
///
/// # Examples
/// ```
/// use std::os::raw::c_char;
///
/// let name = literal_as_c_char!("Test Plugin");
/// // name is now a *const c_char pointing to "Test Plugin"
/// ```
macro_rules! literal_as_c_char {
    ($s:expr) => {
        concat!($s, "\0").as_ptr() as *const c_char
    };
}

#[repr(C)]
pub struct PluginInfo {
    pub name: *const c_char,
    pub version: *const c_char,
    pub description: *const c_char,
    pub author: *const c_char,
    pub default_prefix: *const c_char,
}

#[allow(dead_code)]
#[derive(Clone)]
struct AppInfo {
    name: String,
    description: Option<String>,
    path: String,
    icon: Option<String>,
    emoji: Option<String>,
}

#[repr(C)]
pub struct Entry {
    pub name: *const c_char,        // the display name
    pub description: *const c_char, // still not sure what ill use this for, optional
    pub value: *const c_char,       // the value that is gonna be passed to `handle_selection`
    pub icon: *const c_char,        // icon path (can be null)
    pub emoji: *const c_char,       // emoji (can be null)
}

unsafe impl Send for Entry {}
unsafe impl Sync for Entry {}

#[repr(C)]
pub struct EntryList {
    pub entries: *const Entry,
    pub length: usize,
}

unsafe impl Send for PluginInfo {}
unsafe impl Sync for PluginInfo {}

#[unsafe(no_mangle)]
pub static PLUGIN_INFO: PluginInfo = PluginInfo {
    name: literal_as_c_char!("Clipboard Manager"),
    version: literal_as_c_char!("1.0.1"),
    description: literal_as_c_char!("A plugin for managing your clipboard"),
    author: literal_as_c_char!("Ri"),
    default_prefix: literal_as_c_char!("c"),
};

#[unsafe(no_mangle)]
pub extern "C" fn handle_selection(selection: *const c_char) -> bool {
    if selection.is_null() {
        println!("Selection is null");
        return false;
    }
    
    let selection = unsafe { CStr::from_ptr(selection) };
    let selection_str = selection.to_str().unwrap_or("");
    if selection_str.is_empty() {
        return false;
    }
    println!("Selection: {selection_str:?}");
    match Command::new("sh")
        .arg("-c")
        .arg(format!("cliphist decode {} | wl-copy", selection_str))
        .status() {
        Ok(status) => status.success(),
        Err(e) => {
            println!("Failed to set clipboard content: {}", e);
            false
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_entries() -> EntryList {
    let mut entries: Vec<Entry> = vec![];
    for item in get_clipboard_history() {
        if item.is_empty() {
            continue;
        }
        let split: Vec<&str> = item.splitn(2, '\t').collect();
        if split.len() != 2 {
            continue;
        }
        let id = split[0];
        let content = split[1];
        let id = Box::leak(format!("{}\0", id).into_boxed_str());
        let content = Box::leak(format!("{}\0", content).into_boxed_str());
        entries.push(Entry {
            name: content.as_ptr() as *const c_char,
            description: content.as_ptr() as *const c_char,
            value: id.as_ptr() as *const c_char,
            icon: std::ptr::null(),
            emoji: std::ptr::null(),
        });
    }
    
    let boxed_entries = Box::new(entries);
    let entries_ptr = Box::leak(boxed_entries);
    
    EntryList {
        entries: entries_ptr.as_ptr(),
        length: entries_ptr.len(),
    }
}

fn get_clipboard_history() -> Vec<String> {
    let output = Command::new("cliphist")
        .arg("list")
        .output()
        .expect("Failed to execute cliphist");
    let history = String::from_utf8(output.stdout).unwrap();
    let mut cbh = Vec::new();
    for line in history.split('\n') {
        cbh.push(line.to_string());
    }
    if cbh.len() > 1 && cbh[0] == "opening db: please store something first" {
        cbh.remove(0);
    }
    cbh
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_clipboard_history() {
        let history = get_clipboard_history();
        println!("{:?}", history);
    }

    #[test]
    fn test_handle_selection() {
        let result = handle_selection(literal_as_c_char!("127"));
        assert!(result);
    }

    #[test]
    fn test_get_entries() {
        let entries = get_entries();
        assert!(entries.length > 0);
    }
}
