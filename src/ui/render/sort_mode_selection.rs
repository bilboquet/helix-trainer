//! Sort mode selection screen rendering

use crate::config::SortMode;
use crate::ui::state::{AppState, TypedScreen};
use ratatui::layout;
use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

/// Render the sort mode selection as a centered popup overlay
pub(super) fn render_sort_mode_selection(frame: &mut Frame, state: &AppState) {
    let TypedScreen::SortModeSelection(selection_data) = &state.screen else {
        return;
    };

    let area = frame.area();

    // Calculate popup dimensions based on content
    let modes = SortMode::all();
    // Height: 2 lines per mode (name + description) + 1 top padding + 1 spacing + 1 instructions
    let content_height = (modes.len() as u16 * 2) + 3;
    // Add 2 for popup borders
    let popup_height = (content_height + 2).min(area.height.saturating_sub(4));
    let popup_width = 60u16;

    // Constrain popup dimensions to terminal size
    let popup_width = popup_width.min(area.width.saturating_sub(4));
    let popup_height = popup_height.min(area.height.saturating_sub(4));

    // Create centered popup area
    let popup_area = super::helpers::centered_popup(area, popup_width, popup_height);

    // Clear popup area with black background
    let clear_block = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(clear_block, popup_area);

    // Render popup block with title
    let block = super::helpers::popup_block(Some(" SORT MODE SELECTION "), Color::Cyan);
    frame.render_widget(&block, popup_area);

    // Get inner area for content
    let inner_area = super::helpers::inner_rect(popup_area);

    // Render content inside popup
    render_popup_content(frame, state, selection_data.selected_index, inner_area);
}

/// Render popup content: sort mode list and instructions
fn render_popup_content(
    frame: &mut Frame,
    state: &AppState,
    selected_index: usize,
    area: layout::Rect,
) {
    let modes = SortMode::all();
    let current_mode = state.config.sort_mode;

    if modes.is_empty() {
        let empty_msg = Paragraph::new("No sort modes available")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        frame.render_widget(empty_msg, area);
        return;
    }

    let mut lines = vec![];
    lines.push(Line::from("")); // Top padding

    for (idx, &mode) in modes.iter().enumerate() {
        let is_selected = idx == selected_index;
        let is_current = mode == current_mode;

        // Build the indicator: [*] for current mode, [ ] for others
        let indicator = if is_current { "[*]" } else { "[ ]" };

        // Build the line with highlighting for selected item
        let (prefix, base_style) = if is_selected {
            (
                " > ",
                Style::default()
                    .bg(super::SELECTION_BG_COLOR)
                    .fg(Color::White),
            )
        } else {
            ("   ", Style::default().fg(Color::White))
        };

        let name = mode.display_name();
        let description = mode.description();

        // Indicator color: green if current, gray if not
        let indicator_style = if is_current {
            base_style.fg(Color::Green)
        } else {
            base_style.fg(Color::DarkGray)
        };

        // Name line
        lines.push(Line::from(vec![
            Span::styled(prefix, base_style),
            Span::styled(format!("{} ", indicator), indicator_style),
            Span::styled(name, base_style),
        ]));

        // Description line (indented)
        lines.push(Line::from(vec![
            Span::raw("      "),
            Span::styled(description, Style::default().fg(Color::DarkGray)),
        ]));
    }

    // Add compact instructions at bottom
    lines.push(Line::from("")); // Spacing
    lines.push(Line::from(vec![Span::styled(
        "   j/k: Navigate  |  Enter: Select  |  Esc: Back",
        Style::default().fg(Color::DarkGray),
    )]));

    let paragraph = Paragraph::new(lines)
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::White));
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SortMode;
    use crate::gamification::{ProfileStorage, UserProfile};
    use crate::learning::PerformanceTracker;
    use crate::ui::state::{
        AppState, ConfigState, GameState, ProgressState, SortModeSelectionData, TypedScreen,
        UIState,
    };
    use ratatui::{Terminal, backend::TestBackend};

    fn create_test_state() -> AppState {
        let mut state = AppState {
            screen: TypedScreen::SortModeSelection(SortModeSelectionData::default()),
            ui: UIState::new(),
            game: GameState::new(vec![]),
            progress: ProgressState::new(
                UserProfile::new(),
                PerformanceTracker::new(),
                ProfileStorage::new(),
            ),
            config: ConfigState::default(),
        };
        state.screen = TypedScreen::SortModeSelection(SortModeSelectionData::default());
        state
    }

    #[test]
    fn test_render_sort_mode_selection_no_panic() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        let state = create_test_state();
        let result = terminal.draw(|f| render_sort_mode_selection(f, &state));
        assert!(result.is_ok());
    }

    #[test]
    fn test_sort_mode_display_name_all_variants() {
        assert_eq!(SortMode::ByName.display_name(), "By Name");
        assert_eq!(
            SortMode::ByDifficultyThenCategory.display_name(),
            "By Difficulty, then Category"
        );
    }

    #[test]
    fn test_get_all_sort_modes() {
        let modes = SortMode::all();
        assert_eq!(modes.len(), 7);
    }
}
