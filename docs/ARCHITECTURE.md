# Architecture

KORUKY is organized as a Cargo workspace.

- `koruky-core`: shared domain types and application logic.
- `koruky-parser`: command parsing and validation.
- `koruky-tui`: terminal user interface and executable entry point.

Provider integrations should be isolated behind a future provider interface.
Only official or otherwise explicitly authorized integrations are in scope.
The browser remains responsible for account sign-in and media playback unless
a provider offers a supported integration that permits another flow.
