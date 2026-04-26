use std::path::PathBuf;

pub fn resolve_roots(explicit: &[PathBuf]) -> Vec<PathBuf> {
    if explicit.is_empty() {
        if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
            vec![home.join("Desktop")]
        } else {
            vec![std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))]
        }
    } else {
        explicit.to_vec()
    }
}

pub fn resolve_roots_with_current(explicit: &[PathBuf], current: bool) -> Vec<PathBuf> {
    debug_assert!(!current || explicit.is_empty());

    if current {
        vec![std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))]
    } else {
        resolve_roots(explicit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::TempDir;
    use serial_test::serial;

    struct EnvGuard {
        home: Option<String>,
        cwd: PathBuf,
    }

    impl EnvGuard {
        fn new() -> Self {
            Self {
                home: std::env::var("HOME").ok(),
                cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(home) = &self.home {
                unsafe {
                    std::env::set_var("HOME", home);
                }
            } else {
                unsafe {
                    std::env::remove_var("HOME");
                }
            }

            let _ = std::env::set_current_dir(&self.cwd);
        }
    }

    #[test]
    fn resolve_roots_returns_explicit_roots_when_non_empty() {
        let explicit = vec![PathBuf::from("/tmp/a"), PathBuf::from("/tmp/b")];
        assert_eq!(resolve_roots(&explicit), explicit);
    }

    #[test]
    #[serial]
    fn resolve_roots_uses_home_desktop_when_explicit_empty() {
        let _guard = EnvGuard::new();
        let temp_home = TempDir::new().expect("temp home is created");

        unsafe {
            std::env::set_var("HOME", temp_home.path());
        }

        let roots = resolve_roots(&[]);
        assert_eq!(roots, vec![temp_home.path().join("Desktop")]);
    }

    #[test]
    #[serial]
    fn resolve_roots_with_current_prefers_current_dir() {
        let _guard = EnvGuard::new();
        let temp = TempDir::new().expect("temp directory is created");
        std::env::set_current_dir(temp.path()).expect("cwd is set");

        let roots = resolve_roots_with_current(&[], true);
        let expected = std::env::current_dir().expect("cwd resolves");
        assert_eq!(roots, vec![expected]);
    }

    #[test]
    #[serial]
    fn resolve_roots_falls_back_to_cwd_when_home_unset() {
        let _guard = EnvGuard::new();
        let temp = TempDir::new().expect("temp directory is created");
        std::env::set_current_dir(temp.path()).expect("cwd is set");

        unsafe {
            std::env::remove_var("HOME");
        }

        let roots = resolve_roots(&[]);
        let expected = std::env::current_dir().expect("cwd resolves");
        assert_eq!(roots, vec![expected]);
    }
}
