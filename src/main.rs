use zellij_tile::prelude::*;

use std::collections::BTreeMap;

#[derive(Default)]
struct State {
    marked_panes: Vec<PaneId>,
    selected_index: usize,
    keybinds: Keybinds,
    mock_data: BTreeMap<PaneId, String>
}

register_plugin!(State);

// NOTE: you can start a development environment inside Zellij by running `zellij -l zellij.kdl` in
// this plugin's folder
//
// More info on plugins: https://zellij.dev/documentation/plugins

impl ZellijPlugin for State {
    fn load(&mut self, configuration: BTreeMap<String, String>) {
        self.mock_data.insert(PaneId::Terminal(1), "Terminal 1 title".to_owned());
        self.mock_data.insert(PaneId::Terminal(2), "Terminal 2 title".to_owned());
        self.marked_panes.push(PaneId::Terminal(1));
        self.marked_panes.push(PaneId::Terminal(2));
        subscribe(&[EventType::Key]);
    }
    fn update(&mut self, event: Event) -> bool {
        let mut should_render = false;
        match event {
            Event::Key(key) => {
                match key.bare_key {
                    BareKey::Down if key.has_no_modifiers() => {
                        if self.selected_index + 1 < self.marked_panes.len() {
                            self.selected_index += 1;
                            should_render = true;
                        }
                    }
                    BareKey::Up if key.has_no_modifiers() => {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                            should_render = true;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        should_render
    }
    fn pipe (&mut self, pipe_message: PipeMessage) -> bool {
        let mut should_render = false;
        // react to data piped to this plugin from the CLI, a keybinding or another plugin
        // read more about pipes: https://zellij.dev/documentation/plugin-pipes
        // return true if this plugin's `render` function should be called for the plugin to render
        // itself
        should_render
    }
    fn render(&mut self, rows: usize, cols: usize) {
        let (title, title_width) = self.render_title(cols);
        let (mut explanation_text_lines, explanation_text_width) = self.render_explanation_text(cols);
        let (help, help_text_width) = self.render_help_text(cols);

        let mut lengths = vec![title_width, explanation_text_width, help_text_width];
        lengths.sort();
        let longest_line_count = lengths.last().copied().unwrap_or(0);

        let (mut marked_panes, marked_panes_width) = self.render_marked_panes(longest_line_count, cols);
        let longest_line_count = std::cmp::max(longest_line_count, marked_panes_width);

        let item_count = std::cmp::max(self.marked_panes.iter().count(), 1);
        let base_y = rows.saturating_sub(item_count + 7) / 2;
        let base_x = cols.saturating_sub(longest_line_count) / 2;
        print_text_with_coordinates(title, base_x + longest_line_count.saturating_sub(title_width) / 2, base_y, Some(cols), None);
        for (i, line) in explanation_text_lines.drain(..).enumerate() {
            print_text_with_coordinates(line, base_x, base_y + i + 2, None, None); // this is a
        }
        for (i, text_item) in marked_panes.drain(..).enumerate() {
            print_text_with_coordinates(text_item, base_x, base_y + 5 + i, Some(longest_line_count), None);
        }
        print_text_with_coordinates(help, base_x, base_y + item_count + 6, None, None);
    }
}

impl State {
    fn render_title(&self, _max_width: usize) -> (Text, usize) {
        // here we ignore max width because the title is quite short as is...
        let title_text = "CAROUSEL";
        let title = Text::new(title_text).color_range(2, ..);
        (title, title_text.chars().count())
    }
    fn render_help_text(&self, max_width: usize) -> (Text, usize) {
        let help_text_full = "Help: <ENTER> - focus selected, <0-9> - focus index, <↓↑> - navigate, <Del> - delete selected, <ESC> - hide";
        let help_text_short = "<ENTER/0-9> - focus selected/index, <↓↑/ESC> - navigate/hide, <Del> - delete";
        if help_text_full.chars().count() <= max_width {
            let own_width = help_text_full.chars().count();
            let help = Text::new(help_text_full)
                .color_range(3, 6..=12)
                .color_range(3, 32..=36)
                .color_range(3, 53..=56)
                .color_range(3, 70..=74)
                .color_range(3, 95..=100);
            (help, own_width)
        } else {
            let own_width = help_text_short.chars().count();
            let help = Text::new(help_text_short)
                .color_range(3, ..=10)
                .color_range(3, 36..=43)
                .color_range(3, 62..=66);
            (help, own_width)
        }
    }
    fn render_explanation_text(&self, max_width: usize) -> (Vec<Text>, usize) {
        let mut explanation_text = vec![];
        let mut own_width = 0;
        let mark_pane_shortcut = self.keybinds.mark_pane_shortcut.to_string();
        let show_self_shortcut = self.keybinds.show_self_shortcut.to_string();
        let explanation_text1 = (
            format!("Press <{}> while focused on any pane to bookmark it.", &mark_pane_shortcut),
            format!("<{}> bookmark focused pane.", &mark_pane_shortcut),
        );
        let explanation_text2 = (
            format!("Press <{}> to show this list.", &show_self_shortcut),
            format!("<{}> show this list.", &show_self_shortcut),
        );
        let mut fit_to_width = |texts: (String, String), shortcut_len: usize| {
            if texts.0.chars().count() <= max_width {
                own_width = std::cmp::max(own_width, texts.0.chars().count());
                explanation_text.push(Text::new(texts.0).color_range(3, 6..=6 + shortcut_len + 1));
            } else {
                own_width = std::cmp::max(own_width, texts.1.chars().count());
                explanation_text.push(Text::new(texts.1).color_range(3, ..=shortcut_len + 1));
            }
        };
        fit_to_width(explanation_text1, mark_pane_shortcut.chars().count());
        fit_to_width(explanation_text2, show_self_shortcut.chars().count());
        (explanation_text, own_width)
    }
    fn render_marked_panes(&self, current_width: usize, max_width: usize) -> (Vec<Text>, usize) {
        let mut longest_line_count = current_width;
        let mut text_items: Vec<Text> = vec![];
        for (i, pane_id) in self.marked_panes.iter().enumerate() {
            let (item, item_width) = self.render_list_item(pane_id, max_width, i);
            longest_line_count = std::cmp::max(longest_line_count, item_width);
            text_items.push(item);
        }
        if text_items.is_empty() {
            (vec![Text::new("NO ITEMS.").color_range(0, ..)], longest_line_count)
        } else {
            (text_items, longest_line_count)
        }
    }
    fn render_list_item(&self, pane_id: &PaneId, max_width: usize, i: usize) -> (Text, usize) {
        let mut pane_title = self
            .mock_data
            .get(&pane_id)
            .map(|p| p.as_str())
            .unwrap_or("<UNKNOWN>")
            .to_owned();
        let shortcut_len_and_padding = 4;
        let truncation_len = 4; // this should be 3, but due to an issue with Zellij we need to
                                // make it 4
        if pane_title.chars().count() + truncation_len > max_width.saturating_sub(shortcut_len_and_padding) {
            pane_title.truncate(max_width.saturating_sub(shortcut_len_and_padding + truncation_len));
            pane_title = format!("{}...", pane_title);
        };
        let list_item_text = format!("<{i}> {}", pane_title);
        let mut list_item = Text::new(&list_item_text).color_range(0, ..).color_range(3, ..=3);
        if i == self.selected_index {
            list_item = list_item.selected();
        }
        (list_item, list_item_text.chars().count())
    }
}

struct Keybinds {
    bound_key: bool,
    mark_pane_shortcut: KeyWithModifier,
    show_self_shortcut: KeyWithModifier,
}

impl Default for Keybinds {
    fn default() -> Keybinds {
        Keybinds {
            bound_key: Default::default(),
            mark_pane_shortcut: KeyWithModifier::new(BareKey::Char('i')).with_ctrl_modifier().with_shift_modifier(),
            show_self_shortcut: KeyWithModifier::new(BareKey::Char('o')).with_ctrl_modifier().with_shift_modifier(),
        }
    }
}
