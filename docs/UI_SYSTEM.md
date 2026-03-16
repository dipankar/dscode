# UI System Design

## Table of Contents
- [Overview](#overview)
- [Layout System](#layout-system)
- [Custom Widgets](#custom-widgets)
- [Theme System](#theme-system)
- [Rendering Pipeline](#rendering-pipeline)
- [Extension UI Contributions](#extension-ui-contributions)

## Overview

DSCode's UI is built entirely with **egui** (immediate mode GUI), providing:

- **Native performance**: 60 FPS minimum on all platforms
- **100% VS Code UI parity**: Every panel, widget, and interaction
- **Extensible**: Extensions can contribute custom UI
- **Cross-platform**: Identical on Linux, macOS, Windows
- **Accessible**: Keyboard navigation, screen reader support

## Layout System

### Main Layout Structure

```
┌──────────────────────────────────────────────────────────────┐
│ Title Bar (custom, draggable)                                │
├──┬─────────────────────────────────────────────────────────┬─┤
│A │ ┌─────────────────────────────────────────────────────┐ │P│
│c │ │ Tabs: file1.rs | file2.rs [+]                       │ │a│
│t │ ├─────────────────────────────────────────────────────┤ │n│
│i │ │ Breadcrumbs: workspace > src > file1.rs             │ │e│
│v │ ├─────────────────────────────────────────────────────┤ │l│
│i │ │                                                       │ │ │
│t │ │ Editor Group 1                                       │ │A│
│y │ │ (with line numbers, gutter, minimap)                │ │r│
│  │ │                                                       │ │e│
│B │ ├──────────────── split ────────────────────────────  │ │a│
│a │ │                                                       │ │ │
│r │ │ Editor Group 2                                       │ │ │
│  │ │                                                       │ │ │
│  │ └─────────────────────────────────────────────────────┘ │ │
│  │ ┌─────────────────────────────────────────────────────┐ │ │
│  │ │ Panel: Terminal [v] Problems Output Debug Console   │ │ │
│  │ │ $ cargo build                                        │ │ │
│  │ └─────────────────────────────────────────────────────┘ │ │
└──┴─────────────────────────────────────────────────────────┴─┘
│ Status Bar: Ln 42, Col 16 | UTF-8 | Rust | ✓ main           │
└──────────────────────────────────────────────────────────────┘
```

### Layout Implementation

```rust
pub struct DSCodeLayout {
    /// Title bar (custom window decorations)
    title_bar: TitleBar,

    /// Activity bar (left/right)
    activity_bar: ActivityBar,

    /// Primary sidebar (Explorer, Search, SCM, etc.)
    sidebar: Sidebar,

    /// Editor area (supports grid layout)
    editor_area: EditorArea,

    /// Panel area (Terminal, Problems, Output, etc.)
    panel_area: PanelArea,

    /// Secondary sidebar (optional, right side)
    secondary_sidebar: Option<Sidebar>,

    /// Status bar (bottom)
    status_bar: StatusBar,

    /// Splitter positions (for resizing)
    splitters: Vec<Splitter>,
}

impl DSCodeLayout {
    pub fn render(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Custom title bar (if not using OS decorations)
            if self.use_custom_title_bar {
                self.title_bar.render(ui);
            }

            // Main layout
            egui::TopBottomPanel::bottom("status_bar")
                .resizable(false)
                .show_inside(ui, |ui| {
                    self.status_bar.render(ui);
                });

            // Panel area (bottom/right/left based on config)
            if self.panel_area.visible {
                match self.panel_area.position {
                    PanelPosition::Bottom => {
                        egui::TopBottomPanel::bottom("panel_area")
                            .resizable(true)
                            .show_inside(ui, |ui| {
                                self.panel_area.render(ui);
                            });
                    }
                    PanelPosition::Right => {
                        egui::SidePanel::right("panel_area")
                            .resizable(true)
                            .show_inside(ui, |ui| {
                                self.panel_area.render(ui);
                            });
                    }
                    PanelPosition::Left => { /* similar */ }
                }
            }

            // Activity bar
            let activity_panel = if self.activity_bar.position == Position::Left {
                egui::SidePanel::left("activity_bar")
            } else {
                egui::SidePanel::right("activity_bar")
            };

            activity_panel
                .resizable(false)
                .default_width(48.0)
                .show_inside(ui, |ui| {
                    self.activity_bar.render(ui);
                });

            // Primary sidebar
            if self.sidebar.visible {
                let sidebar_panel = if self.sidebar.position == Position::Left {
                    egui::SidePanel::left("sidebar")
                } else {
                    egui::SidePanel::right("sidebar")
                };

                sidebar_panel
                    .resizable(true)
                    .default_width(250.0)
                    .show_inside(ui, |ui| {
                        self.sidebar.render(ui);
                    });
            }

            // Central editor area
            egui::CentralPanel::default().show_inside(ui, |ui| {
                self.editor_area.render(ui);
            });
        });
    }
}
```

## Custom Widgets

### 1. Editor Widget

The most critical widget - displays text with syntax highlighting, cursors, selections, etc.

```rust
pub struct EditorWidget {
    /// Text buffer (rope)
    buffer: Arc<RwLock<Rope>>,

    /// Syntax tree (tree-sitter)
    syntax: Option<Tree>,

    /// Theme
    theme: Arc<Theme>,

    /// Cursors and selections
    cursors: Vec<Cursor>,

    /// Viewport (visible lines)
    viewport: Viewport,

    /// Scroll position
    scroll: Vec2,

    /// Font
    font: FontId,
}

impl EditorWidget {
    pub fn render(&mut self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();

        // Calculate visible lines
        let line_height = ui.fonts(|f| f.row_height(&self.font));
        let visible_lines = (available_size.y / line_height).ceil() as usize;
        let start_line = (self.scroll.y / line_height) as usize;
        let end_line = start_line + visible_lines + 1;

        // Prepare painter
        let (response, painter) = ui.allocate_painter(available_size, egui::Sense::click_and_drag());

        // Get visible text
        let buffer = self.buffer.read().unwrap();
        let visible_text = buffer.lines(start_line..end_line.min(buffer.len_lines()));

        // Render line numbers
        let gutter_width = self.render_gutter(&painter, start_line, end_line, line_height);

        // Render text with syntax highlighting
        let text_offset = egui::pos2(gutter_width, 0.0);
        self.render_text(
            &painter,
            &visible_text,
            text_offset,
            line_height,
            start_line,
        );

        // Render cursors
        self.render_cursors(&painter, text_offset, line_height, start_line);

        // Render selections
        self.render_selections(&painter, text_offset, line_height, start_line);

        // Render minimap (if enabled)
        if self.show_minimap {
            self.render_minimap(ui, &buffer);
        }

        // Handle input
        if response.clicked() {
            let click_pos = response.interact_pointer_pos().unwrap();
            self.handle_click(click_pos, line_height, gutter_width);
        }

        if response.dragged() {
            self.handle_drag(response.drag_delta());
        }
    }

    fn render_text(
        &self,
        painter: &egui::Painter,
        text: &str,
        offset: Pos2,
        line_height: f32,
        start_line: usize,
    ) {
        // Get syntax highlighting from tree-sitter
        let highlights = self.get_highlights(text, start_line);

        for (line_idx, line) in text.lines().enumerate() {
            let y = offset.y + (line_idx as f32 * line_height);

            // Render line with syntax highlighting
            let mut x = offset.x;
            for token in &highlights[line_idx] {
                let color = self.theme.get_token_color(&token.scope);
                let text = &line[token.range.clone()];

                painter.text(
                    egui::pos2(x, y),
                    egui::Align2::LEFT_TOP,
                    text,
                    self.font.clone(),
                    color,
                );

                x += painter.fonts(|f| f.glyph_width(&self.font, text));
            }
        }
    }

    fn render_gutter(&self, painter: &egui::Painter, start_line: usize, end_line: usize, line_height: f32) -> f32 {
        let gutter_width = 60.0; // Enough for line numbers + git decorations + breakpoints

        // Background
        painter.rect_filled(
            egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(gutter_width, painter.clip_rect().height()),
            ),
            0.0,
            self.theme.gutter_bg,
        );

        // Line numbers
        for line_num in start_line..end_line {
            let y = (line_num - start_line) as f32 * line_height;

            // Git decorations (added, modified, deleted)
            if let Some(git_status) = self.get_git_status(line_num) {
                let color = match git_status {
                    GitStatus::Added => self.theme.git_added,
                    GitStatus::Modified => self.theme.git_modified,
                    GitStatus::Deleted => self.theme.git_deleted,
                };

                painter.rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(0.0, y),
                        egui::vec2(3.0, line_height),
                    ),
                    0.0,
                    color,
                );
            }

            // Breakpoint indicator
            if self.has_breakpoint(line_num) {
                painter.circle_filled(
                    egui::pos2(20.0, y + line_height / 2.0),
                    6.0,
                    self.theme.breakpoint_color,
                );
            }

            // Line number
            painter.text(
                egui::pos2(35.0, y),
                egui::Align2::RIGHT_TOP,
                format!("{}", line_num + 1),
                self.font.clone(),
                self.theme.line_number_fg,
            );
        }

        gutter_width
    }

    fn render_minimap(&self, ui: &mut egui::Ui, buffer: &Rope) {
        let minimap_width = 100.0;

        egui::SidePanel::right("minimap")
            .exact_width(minimap_width)
            .show_inside(ui, |ui| {
                let (response, painter) = ui.allocate_painter(
                    egui::vec2(minimap_width, ui.available_height()),
                    egui::Sense::click(),
                );

                // Render entire document at small scale
                let scale = ui.available_height() / (buffer.len_lines() as f32);
                let char_width = 1.0;

                for (line_idx, line) in buffer.lines(..).enumerate() {
                    let y = line_idx as f32 * scale;

                    // Simple rendering: just show text density
                    for (char_idx, ch) in line.chars().enumerate() {
                        if !ch.is_whitespace() {
                            let x = char_idx as f32 * char_width;
                            painter.rect_filled(
                                egui::Rect::from_min_size(
                                    egui::pos2(x, y),
                                    egui::vec2(char_width, scale),
                                ),
                                0.0,
                                self.theme.minimap_char_color,
                            );
                        }
                    }
                }

                // Show viewport rectangle
                let viewport_rect = egui::Rect::from_min_size(
                    egui::pos2(0.0, self.viewport.start_line as f32 * scale),
                    egui::vec2(minimap_width, self.viewport.visible_lines as f32 * scale),
                );

                painter.rect_stroke(
                    viewport_rect,
                    0.0,
                    egui::Stroke::new(1.0, self.theme.minimap_viewport_color),
                );

                // Handle click to scroll
                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let clicked_line = (pos.y / scale) as usize;
                        self.scroll_to_line(clicked_line);
                    }
                }
            });
    }
}
```

### 2. Tab Bar Widget

```rust
pub struct TabBar {
    tabs: Vec<Tab>,
    active_index: usize,
    pinned_indices: HashSet<usize>,
}

impl TabBar {
    pub fn render(&mut self, ui: &mut egui::Ui) -> TabBarResponse {
        ui.horizontal(|ui| {
            let mut response = TabBarResponse::default();

            for (idx, tab) in self.tabs.iter().enumerate() {
                let is_active = idx == self.active_index;
                let is_pinned = self.pinned_indices.contains(&idx);

                let tab_response = self.render_tab(ui, tab, is_active, is_pinned);

                if tab_response.clicked {
                    response.activated = Some(idx);
                }

                if tab_response.close_clicked {
                    response.closed = Some(idx);
                }

                if tab_response.dragged {
                    response.dragged = Some((idx, tab_response.drag_delta));
                }

                // Context menu
                tab_response.response.context_menu(|ui| {
                    if ui.button("Close").clicked() {
                        response.closed = Some(idx);
                        ui.close_menu();
                    }

                    if ui.button("Close Others").clicked() {
                        response.close_others = Some(idx);
                        ui.close_menu();
                    }

                    if ui.button(if is_pinned { "Unpin" } else { "Pin" }).clicked() {
                        response.pin_toggled = Some(idx);
                        ui.close_menu();
                    }

                    if ui.button("Split Right").clicked() {
                        response.split = Some((idx, SplitDirection::Right));
                        ui.close_menu();
                    }
                });
            }

            // "+" button to open new file
            if ui.button("+").clicked() {
                response.new_tab = true;
            }

            response
        })
        .inner
    }

    fn render_tab(&self, ui: &mut egui::Ui, tab: &Tab, is_active: bool, is_pinned: bool) -> TabResponse {
        let bg_color = if is_active {
            self.theme.tab_active_bg
        } else {
            self.theme.tab_inactive_bg
        };

        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(if is_pinned { 40.0 } else { 120.0 }, 35.0),
            egui::Sense::click_and_drag(),
        );

        // Background
        ui.painter().rect_filled(rect, 0.0, bg_color);

        // Icon (if pinned, only show icon)
        let icon_pos = rect.min + egui::vec2(8.0, 8.0);
        ui.painter().text(
            icon_pos,
            egui::Align2::LEFT_TOP,
            &tab.icon,
            FontId::default(),
            self.theme.icon_color,
        );

        if !is_pinned {
            // Label (truncated)
            let label_pos = icon_pos + egui::vec2(20.0, 0.0);
            let label = truncate_text(&tab.label, 80.0);
            ui.painter().text(
                label_pos,
                egui::Align2::LEFT_TOP,
                label,
                FontId::default(),
                self.theme.tab_fg,
            );

            // Dirty indicator
            if tab.is_dirty {
                ui.painter().circle_filled(
                    label_pos + egui::vec2(-10.0, 5.0),
                    4.0,
                    self.theme.dirty_indicator,
                );
            }

            // Close button
            let close_rect = egui::Rect::from_min_size(
                rect.max - egui::vec2(25.0, 25.0),
                egui::vec2(20.0, 20.0),
            );

            let close_response = ui.interact(close_rect, ui.id().with("close"), egui::Sense::click());

            if close_response.hovered() || is_active {
                ui.painter().text(
                    close_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "✕",
                    FontId::default(),
                    self.theme.close_button_fg,
                );
            }

            if close_response.clicked() {
                return TabResponse {
                    response,
                    close_clicked: true,
                    ..Default::default()
                };
            }
        }

        TabResponse {
            response,
            clicked: response.clicked(),
            dragged: response.dragged(),
            drag_delta: response.drag_delta(),
            ..Default::default()
        }
    }
}
```

### 3. Tree View Widget

```rust
pub struct TreeView {
    root: TreeNode,
    expanded: HashSet<String>,
    selected: Option<String>,
}

impl TreeView {
    pub fn render(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.render_node(ui, &self.root, 0);
        });
    }

    fn render_node(&mut self, ui: &mut egui::Ui, node: &TreeNode, depth: usize) {
        let indent = depth as f32 * 20.0;

        ui.horizontal(|ui| {
            ui.add_space(indent);

            // Expand/collapse icon
            if !node.children.is_empty() {
                let icon = if self.expanded.contains(&node.id) {
                    "▼"
                } else {
                    "▶"
                };

                if ui.button(icon).clicked() {
                    if self.expanded.contains(&node.id) {
                        self.expanded.remove(&node.id);
                    } else {
                        self.expanded.insert(node.id.clone());
                    }
                }
            } else {
                ui.add_space(20.0); // Spacing for leaf nodes
            }

            // Icon
            ui.label(&node.icon);

            // Label
            let is_selected = self.selected.as_ref() == Some(&node.id);
            let label_response = ui.selectable_label(is_selected, &node.label);

            if label_response.clicked() {
                self.selected = Some(node.id.clone());
            }

            // Context menu
            label_response.context_menu(|ui| {
                if ui.button("Rename").clicked() {
                    // Handle rename
                    ui.close_menu();
                }

                if ui.button("Delete").clicked() {
                    // Handle delete
                    ui.close_menu();
                }
            });
        });

        // Render children if expanded
        if self.expanded.contains(&node.id) {
            for child in &node.children {
                self.render_node(ui, child, depth + 1);
            }
        }
    }
}
```

### 4. Command Palette

```rust
pub struct CommandPalette {
    query: String,
    results: Vec<CommandItem>,
    selected_index: usize,
}

impl CommandPalette {
    pub fn render(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("Command Palette")
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 100.0))
            .fixed_size(egui::vec2(600.0, 400.0))
            .show(ui.ctx(), |ui| {
                // Search box
                let search_response = ui.text_edit_singleline(&mut self.query);

                if search_response.changed() {
                    self.update_results();
                }

                // Results list
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (idx, item) in self.results.iter().enumerate() {
                        let is_selected = idx == self.selected_index;

                        let response = ui.selectable_label(is_selected, format!("{} - {}", item.label, item.keybinding));

                        if response.clicked() {
                            self.execute_command(&item.id);
                        }

                        if response.hovered() {
                            self.selected_index = idx;
                        }
                    }
                });

                // Keyboard navigation
                ui.input(|i| {
                    if i.key_pressed(egui::Key::ArrowDown) {
                        self.selected_index = (self.selected_index + 1).min(self.results.len() - 1);
                    }

                    if i.key_pressed(egui::Key::ArrowUp) {
                        self.selected_index = self.selected_index.saturating_sub(1);
                    }

                    if i.key_pressed(egui::Key::Enter) {
                        if let Some(item) = self.results.get(self.selected_index) {
                            self.execute_command(&item.id);
                        }
                    }
                });
            });
    }

    fn update_results(&mut self) {
        // Fuzzy search
        let query = self.query.to_lowercase();
        self.results = ALL_COMMANDS
            .iter()
            .filter(|cmd| {
                cmd.label.to_lowercase().contains(&query)
                    || cmd.id.to_lowercase().contains(&query)
            })
            .take(50)
            .cloned()
            .collect();

        self.selected_index = 0;
    }
}
```

## Theme System

### Theme Structure

```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Base colors
    pub editor_bg: Color32,
    pub editor_fg: Color32,
    pub gutter_bg: Color32,
    pub line_number_fg: Color32,

    /// Selection
    pub selection_bg: Color32,
    pub selection_fg: Color32,

    /// UI elements
    pub sidebar_bg: Color32,
    pub titlebar_bg: Color32,
    pub statusbar_bg: Color32,
    pub tab_active_bg: Color32,
    pub tab_inactive_bg: Color32,

    /// Syntax highlighting
    pub syntax_colors: HashMap<String, Color32>,

    /// Git
    pub git_added: Color32,
    pub git_modified: Color32,
    pub git_deleted: Color32,

    /// Semantic colors
    pub error_fg: Color32,
    pub warning_fg: Color32,
    pub info_fg: Color32,
}

impl Theme {
    pub fn from_vscode_theme(json: &str) -> Result<Self> {
        // Parse VS Code theme JSON
        let vscode_theme: VSCodeTheme = serde_json::from_str(json)?;

        // Convert to DSCode theme
        Ok(Self {
            editor_bg: parse_color(&vscode_theme.colors["editor.background"]),
            editor_fg: parse_color(&vscode_theme.colors["editor.foreground"]),
            // ... map all colors
            syntax_colors: vscode_theme
                .token_colors
                .into_iter()
                .map(|tc| (tc.scope, parse_color(&tc.settings.foreground)))
                .collect(),
        })
    }
}
```

### VS Code Theme Compatibility

```rust
pub struct ThemeLoader {
    themes: HashMap<String, Theme>,
}

impl ThemeLoader {
    pub fn load_vscode_theme(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let theme = Theme::from_vscode_theme(&content)?;
        let name = path.file_stem().unwrap().to_str().unwrap();
        self.themes.insert(name.to_string(), theme);
        Ok(())
    }

    pub fn scan_vscode_themes(&mut self) {
        // Scan VS Code extension directories for themes
        let vscode_extensions = dirs::home_dir()
            .unwrap()
            .join(".vscode/extensions");

        for entry in walkdir::WalkDir::new(vscode_extensions) {
            let entry = entry.unwrap();
            if entry.path().extension() == Some("json".as_ref())
                && entry.path().to_str().unwrap().contains("themes")
            {
                let _ = self.load_vscode_theme(entry.path());
            }
        }
    }
}
```

## Rendering Pipeline

### Frame Loop

```rust
pub struct DSCodeApp {
    layout: DSCodeLayout,
    theme: Arc<Theme>,
}

impl eframe::App for DSCodeApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Request continuous repaint for animations
        ctx.request_repaint();

        // Update state from IPC messages
        self.poll_ipc_messages();

        // Render layout
        self.layout.render(ctx);

        // Render overlays (notifications, hover, completion, etc.)
        self.render_overlays(ctx);
    }
}
```

### Performance Optimizations

```rust
// Cache layout calculations
pub struct LayoutCache {
    line_heights: Vec<f32>,
    glyph_widths: HashMap<char, f32>,
    token_positions: Vec<Vec<Pos2>>,
}

// Only repaint dirty regions
impl EditorWidget {
    fn mark_dirty(&mut self, range: Range<usize>) {
        self.dirty_lines.insert(range);
    }

    fn render(&mut self, ui: &mut egui::Ui) {
        // Only render dirty lines
        for line_idx in &self.dirty_lines {
            self.render_line(ui, *line_idx);
        }

        self.dirty_lines.clear();
    }
}
```

## Extension UI Contributions

Extensions can contribute UI through:

1. **Tree Views**: Custom sidebars
2. **Webviews**: Full HTML/CSS/JS panels
3. **Status Bar Items**: Custom status information
4. **Commands**: Available in command palette
5. **Menus**: Context menus, editor menus, etc.

See [EXTENSION_SYSTEM.md](EXTENSION_SYSTEM.md) for details.
