# UI modules (`optirust-ui`)

The shell (`optirust-ui`) owns the window: menus, workspace tabs, toolbars, docked panes, canvas, status bar, preferences, shortcuts, command palette, theme, i18n. A workspace is a `Module` in its own crate; `optirust` registers it. Toolkit: `eframe`/`egui` 0.36 with `egui_dock` 0.21 (`eframe::App::ui`, `egui_dock::DockArea::show_inside`, `TabViewer::id`). Spec: `docs/spec/07-ui-shell.md`.

## The contract

```rust sig
pub trait Module {
fn id(&self) -> WorkspaceId;
fn default_layout(&self) -> DockState<PaneId>;
fn menus(&mut self, ui: &mut egui::Ui, shell: &mut ShellCtx);
fn toolbars(&mut self, ui: &mut egui::Ui, shell: &mut ShellCtx);
fn pane_title(&self, pane: PaneId, shell: &ShellCtx) -> egui::WidgetText;
fn pane_ui(&mut self, pane: PaneId, ui: &mut egui::Ui, shell: &mut ShellCtx);
fn status(&self, shell: &ShellCtx) -> StatusLine;
fn actions(&self) -> &[Action];
fn run(&mut self, action: ActionId, shell: &mut ShellCtx);
fn open(&mut self, path: &std::path::Path, shell: &mut ShellCtx) -> Result<bool, optirust_io::IoError>;
fn catalog(&self) -> Option<Catalog> {
fn on_job(&mut self, _event: JobEvent, _shell: &mut ShellCtx) {}
```

- Required: `id`, `default_layout`, `menus`, `toolbars`, `pane_title`, `pane_ui`, `status`, `actions`, `run`, `open`. Optional with defaults: `catalog`, `extend_menu`, `on_file`, `on_job`, `pref_pages`, `pref_ui`.
- `open` returns `true` when the module opened `path`; the shell opens `.optx`, `.optm`, `.optc` itself and offers the rest (e.g. `.dxf`) to modules.
- `run` is called only after the shell checked the action is enabled.
- `WorkspaceId` is a closed enum (`Pds`, `Grading`, `Marker`, `CutPlan`); a fifth workspace means a new variant, `ALL`, `message_id`, `key` and a `layout::` default.

## Steps to add a module

1. New crate `crates/optirust-<name>` depending on `optirust-core`, `optirust-io`, `optirust-ui` (workspace deps). Domain logic and commands go in non-UI modules with their own tests; the `ui/` module only draws and dispatches.
2. Implement `Module` for `<Name>Module` (derive `Default`). Hold the workspace state in the module, never in `ShellCtx`. Edit the active document only through `Command`s (`core-model.md`), so undo and validation work.
3. Declare actions as `const` `Action::new(ActionId("<ws>-<verb>"))` with `.keys(..)`, `.icon(..)`, `.enabled(enabled::needs_style | needs_marker | needs_cut_plan | needs_document | ...)`. The `ActionId` string is also the Fluent message id of its label.
4. Panes are `PaneId("snake_case")` constants; `default_layout` builds a `DockState<PaneId>` with the `layout::` helpers. `pane_ui` matches on the `PaneId`.
5. Strings go through Fluent. Crate `i18n.toml` (`fallback_language = "en"`, `assets_dir = "i18n"`, `domain = "optirust_<name>"`), files `i18n/en/optirust_<name>.ftl` and `i18n/id/optirust_<name>.ftl`. In `src/i18n.rs` define `Localizations` (`RustEmbed`, folder `i18n/`), a static `LOADER` built with `optirust_ui::i18n::new_loader(fluent_language_loader!(), &Localizations)`, and `catalog()` returning `optirust_ui::Catalog::new(&LOADER, &Localizations)`. Return it from `Module::catalog`. UI text uses `fl!` or `Msg::new("message-id")`; English is the source and the `id` file is updated in the same change.
6. Long work (nesting, plot, import, export) runs on `std::thread` + `std::sync::mpsc` through the shell's jobs (`JobEvent` arrives in `on_job`); poll, never block the UI thread; support cancel.
7. Register in `crates/optirust/src/modules.rs`: add `Box::new(optirust_<name>::<Name>Module::default())` to `register_modules()`. A workspace is registered at most once (the file's test checks it).
8. Tests (see `testing.md`): logic tests in the crate; `i18n.rs` tests `assert_parity(&Localizations, "optirust_<name>")` and that every message id the crate looks up exists in `en` and `id`; UI behaviour with `egui_kittest` by label.

Do not use `egui` immediate-mode state to hold document data; panes read the document each frame and emit commands.
