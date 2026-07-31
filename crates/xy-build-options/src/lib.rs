pub mod schema;

define_options! {
    "Root configuration for XY Build projects."
    pub struct Config {
        "The project type being built."
        pub project_type: Enum(ProjectType),
        "The output directory for compiled build artifacts."
        pub output: String,
        "The compilation target triple. If unset, the host target is used."
        pub target: String,
        "Path to the source directory containing input files."
        pub source: String,
        "Path to the directory where intermediate build artifacts are stored."
        pub work: String,
        "Enable verbose logging during the build process."
        pub verbose: Enum(Bool),
        "Enable compiler optimizations."
        pub optimize: Enum(Bool),
        "Configuration for Node.js module resolution."
        pub node_modules: Object(NodeModules),
        "List of dependency specifications, one per nested entry."
        pub dependencies: String,
        "Script commands to run during the build process."
        pub run: Object(Run),
        "A user-defined script command to run at a specific build phase."
        pub script: String,
    }
}

define_options! {
    "The type of project being built."
    pub enum ProjectType {
        "Node.js project"
        Node,
        "Rust project"
        Rust,
        "Zig project"
        Zig,
    }
}

define_options! {
    "Configuration for Node.js module resolution."
    pub struct NodeModules {
        "The package manager to use."
        pub manager: Enum(Manager),
    }
}

define_options! {
    "The package manager to use."
    pub enum Manager {
        "npm"
        Npm,
        "pnpm"
        Pnpm,
    }
}

define_options! {
    "Script commands to run during the build process."
    pub struct Run {
        "The command to execute."
        pub command: String,
        "Arguments for the command."
        pub arguments: Object(Arguments),
    }
}

define_options! {
    "Arguments for a command."
    pub struct Arguments {
        "A file argument."
        pub file: String,
    }
}

define_options! {
    "Boolean value: true or false."
    pub enum Bool {
        "True"
        True,
        "False"
        False,
    }
}

/// Return all top-level field names and their docs (flat, for completion).
pub fn all_options() -> Vec<(&'static str, &'static str)> {
    Config_SCHEMA
        .fields
        .iter()
        .map(|f| (f.name, f.doc))
        .collect()
}
