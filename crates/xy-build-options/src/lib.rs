/// XY Build configuration options.
///
/// This module defines all known configuration keys recognized by the XY Build system.
/// Each constant is documented with its purpose and usage.

/// The output directory for compiled build artifacts.
/// Default: `"build"`
pub const OUTPUT: &str = "output";

/// The compilation target triple (e.g., `"x86_64-unknown-linux-gnu"`, `"wasm32-unknown-unknown"`).
/// If unset, the host target is used.
pub const TARGET: &str = "target";

/// Path to the source directory containing input files.
/// Default: `"src"`
pub const SOURCE_DIR: &str = "source";

/// Path to the directory where intermediate build artifacts are stored.
/// Default: `".xybuild"`
pub const WORK_DIR: &str = "work";

/// Enable verbose logging during the build process.
/// Values: `"true"` or `"false"`.
/// Default: `"false"`
pub const VERBOSE: &str = "verbose";

/// Enable compiler optimizations.
/// Values: `"true"` or `"false"`.
/// Default: `"false"`
pub const OPTIMIZE: &str = "optimize";

/// List of dependency specifications, one per nested entry.
/// Each dependency entry may contain its own sub-options.
pub const DEPENDENCIES: &str = "dependencies";

/// A user-defined script command to run at a specific build phase.
/// The value is a shell command string.
pub const SCRIPT: &str = "script";

/// Returns all known options and their documentation descriptions.
///
/// This is used by the LSP to provide hover documentation and
/// autocompletion suggestions for `.xybuild` files.
pub fn all_options() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            OUTPUT,
            "The output directory for compiled build artifacts. Default: \"build\"",
        ),
        (
            TARGET,
            "The compilation target triple (e.g., \"x86_64-unknown-linux-gnu\"). If unset, the host target is used.",
        ),
        (
            SOURCE_DIR,
            "Path to the source directory containing input files. Default: \"src\"",
        ),
        (
            WORK_DIR,
            "Path to the directory where intermediate build artifacts are stored. Default: \".xybuild\"",
        ),
        (
            VERBOSE,
            "Enable verbose logging during the build process. Values: \"true\" or \"false\". Default: \"false\"",
        ),
        (
            OPTIMIZE,
            "Enable compiler optimizations. Values: \"true\" or \"false\". Default: \"false\"",
        ),
        (
            DEPENDENCIES,
            "List of dependency specifications, one per nested entry. Each dependency entry may contain its own sub-options.",
        ),
        (
            SCRIPT,
            "A user-defined script command to run at a specific build phase. The value is a shell command string.",
        ),
    ]
}
