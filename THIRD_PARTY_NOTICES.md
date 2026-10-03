# Third-party notices

The project is MIT licensed. Direct Rust dependencies:

| Dependency | License | Purpose |
| --- | --- | --- |
| serde 1.0.229 | MIT OR Apache-2.0 | Versioned serialization |
| serde_json 1.0.151 | MIT OR Apache-2.0 | JSON logs and reports |
| sha2 0.10.9 | MIT OR Apache-2.0 | Canonical SHA-256 run hashes |
| Tauri 2.12.1 / tauri-build 2.7.1 | MIT OR Apache-2.0 | Windows app and bundler |
| React / React DOM 19.3.0 | MIT | Product UI |
| @tauri-apps/api 2.12.1 | MIT OR Apache-2.0 | Desktop IPC adapter |

`Cargo.lock`, `src-tauri/Cargo.lock` and `package-lock.json` pin transitive versions. `npm run licenses` collects Windows Rust dependency license declarations and available upstream texts plus frontend runtime license texts into `build-notices/THIRD-PARTY-LICENSES.txt`. The desktop build includes this resource and the project license. Vite, Vitest, ESLint and Playwright are development/test tools, not bundled runtime code.
