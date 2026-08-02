mod schema;
pub use schema::*;

parseable_enum!(ProjectType { Node, Rust });

parseable_struct!(
    // "Root configuration for XY Build projects."
    Config {
        // "The project type being built."
        project_type: ProjectType,
        // "Configuration for Node.js module resolution."
        node_modules: NodeModules,
        // "Script commands to run during the build process."
        run: Run,
    }
);

parseable_struct!(Run { rem: Remainder });

parseable_enum!(NodePackageManagers { Pnpm, Npm });

parseable_struct!(NodeModules {
    package_manager: NodePackageManagers,
    deps: Remainder
});
