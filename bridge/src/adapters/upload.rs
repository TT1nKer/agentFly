use std::fs;
use std::path::Path;

pub fn save_file(workspace: &str, filename: &str, data: &[u8]) -> Result<String, String> {
    let inbox_dir = Path::new(workspace).join("inbox");
    fs::create_dir_all(&inbox_dir)
        .map_err(|e| format!("create inbox dir: {}", e))?;

    let sanitized = sanitize_filename(filename);
    let filepath = inbox_dir.join(&sanitized);

    if filepath.exists() {
        return Err(format!("File {} already exists", sanitized));
    }

    let canonical_inbox = inbox_dir.canonicalize()
        .map_err(|e| format!("canonicalize inbox: {}", e))?;

    fs::write(&filepath, data)
        .map_err(|e| format!("write file: {}", e))?;

    let canonical_file = filepath.canonicalize()
        .map_err(|e| format!("canonicalize file: {}", e))?;

    if !canonical_file.starts_with(&canonical_inbox) {
        let _ = fs::remove_file(&filepath);
        return Err("Path traversal detected".to_string());
    }

    Ok(canonical_file.to_string_lossy().to_string())
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

pub fn validate_upload(media_type: &str, size: usize) -> Result<(), String> {
    match media_type {
        "text" => {
            if size > 100 * 1024 {
                return Err("Text exceeds 100KB limit".to_string());
            }
        }
        "url" => {
            if size > 4 * 1024 {
                return Err("URL exceeds 4KB limit".to_string());
            }
        }
        "image" => {
            if size > 5 * 1024 * 1024 {
                return Err("Image exceeds 5MB limit".to_string());
            }
        }
        "file" => {
            if size > 10 * 1024 * 1024 {
                return Err("File exceeds 10MB limit".to_string());
            }
        }
        _ => return Err(format!("Unknown media type: {}", media_type)),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_text_file() {
        let tmp = std::env::temp_dir().join("ac_upload_test");
        let _ = fs::create_dir_all(&tmp);
        let _ = fs::remove_dir_all(tmp.join("inbox"));

        let path = save_file(tmp.to_str().unwrap(), "test.txt", b"hello world").unwrap();
        assert!(path.contains("inbox"));
        assert!(path.contains("test.txt"));

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "hello world");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_path_traversal_blocked() {
        let tmp = std::env::temp_dir().join("ac_upload_traversal");
        let _ = fs::create_dir_all(&tmp);
        let _ = fs::remove_dir_all(tmp.join("inbox"));

        let result = save_file(tmp.to_str().unwrap(), "../evil.txt", b"bad");
        assert!(result.is_err() || result.unwrap().contains("inbox"));

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_validate_size_limits() {
        assert!(validate_upload("text", 50 * 1024).is_ok());
        assert!(validate_upload("text", 200 * 1024).is_err());
        assert!(validate_upload("url", 5 * 1024).is_err());
        assert!(validate_upload("image", 6 * 1024 * 1024).is_err());
        assert!(validate_upload("file", 11 * 1024 * 1024).is_err());
    }

    #[test]
    fn test_validate_allowed_limits() {
        assert!(validate_upload("text", 100 * 1024).is_ok());
        assert!(validate_upload("url", 4 * 1024).is_ok());
        assert!(validate_upload("image", 5 * 1024 * 1024).is_ok());
        assert!(validate_upload("file", 10 * 1024 * 1024).is_ok());
    }
}
