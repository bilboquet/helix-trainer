//! Sort mode selection screen message handlers
//!
//! Handles navigation and selection operations for the sort mode selection screen

use crate::config::SortMode;
use crate::security::UserError;
use crate::ui::state::{
    HandlerContext, HandlerOutcome, ReturnDestination, SortModeSelectionData, TypedScreen,
};

/// Determine return destination based on current context
///
/// If coming from a paused mini-game, return there; otherwise return to menu.
fn determine_return_destination(ctx: &HandlerContext<'_>) -> ReturnDestination {
    if let Some(session) = &ctx.game.minigame_session
        && session.state().is_paused()
    {
        return ReturnDestination::PausedMiniGame;
    }
    ReturnDestination::Menu
}

/// Handle ShowSortModeSelection message
///
/// Navigates to the sort mode selection screen, tracking where to return
pub fn handle_show_sort_mode_selection(
    ctx: &mut HandlerContext<'_>,
) -> Result<HandlerOutcome, UserError> {
    let return_to = determine_return_destination(ctx);
    let modes = SortMode::all();

    // Find current mode index to pre-select it
    let current_mode = ctx.config.sort_mode;
    let selected_index = modes.iter().position(|&m| m == current_mode).unwrap_or(0);

    Ok(HandlerOutcome::Transition(Box::new(
        TypedScreen::SortModeSelection(SortModeSelectionData {
            selected_index,
            return_to,
        }),
    )))
}

/// Handle SortModeSelectionUp message
///
/// Moves selection up in the sort mode list with wraparound
pub fn handle_sort_mode_selection_up(
    data: &mut SortModeSelectionData,
    _ctx: &HandlerContext<'_>,
) -> Result<HandlerOutcome, UserError> {
    let modes = SortMode::all();
    let mode_count = modes.len();

    if mode_count == 0 {
        return Ok(HandlerOutcome::Stay);
    }

    // Bounds check: ensure selected_index is valid before decrement
    if data.selected_index >= mode_count {
        data.selected_index = mode_count.saturating_sub(1);
    }

    // Move up with wraparound
    if data.selected_index > 0 {
        data.selected_index -= 1;
    } else {
        data.selected_index = mode_count.saturating_sub(1);
    }

    Ok(HandlerOutcome::Stay)
}

/// Handle SortModeSelectionDown message
///
/// Moves selection down in the sort mode list with wraparound
pub fn handle_sort_mode_selection_down(
    data: &mut SortModeSelectionData,
    _ctx: &HandlerContext<'_>,
) -> Result<HandlerOutcome, UserError> {
    let modes = SortMode::all();
    let mode_count = modes.len();

    if mode_count == 0 {
        return Ok(HandlerOutcome::Stay);
    }

    // Bounds check: ensure selected_index is valid
    if data.selected_index >= mode_count {
        data.selected_index = 0;
    }

    // Move down with wraparound
    if data.selected_index < mode_count - 1 {
        data.selected_index += 1;
    } else {
        data.selected_index = 0;
    }

    Ok(HandlerOutcome::Stay)
}

/// Handle SortModeSelectionSelect message
///
/// Applies the selected sort mode and returns to the previous screen
pub fn handle_sort_mode_selection_select(
    data: &SortModeSelectionData,
    ctx: &mut HandlerContext<'_>,
) -> Result<HandlerOutcome, UserError> {
    let modes = SortMode::all();

    // Bounds check: ensure selected_index is valid
    let Some(&selected_mode) = modes.get(data.selected_index) else {
        return Ok(HandlerOutcome::Stay);
    };

    // Apply the selected sort mode
    let profile = &ctx.progress.profile;
    ctx.game
        .scenario_collection
        .sort(selected_mode, Some(profile));
    ctx.config.sort_mode = selected_mode;

    // Stay in the popup to allow selecting another mode or exiting with Esc
    Ok(HandlerOutcome::Stay)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SortMode;
    use crate::gamification::{ProfileStorage, UserProfile};
    use crate::learning::PerformanceTracker;
    use crate::ui::state::{
        ConfigState, GameState, ProgressState, ReturnDestination, TypedScreen, UIState,
    };

    fn create_test_context() -> (UIState, GameState, ProgressState, ConfigState) {
        (
            UIState::new(),
            GameState::new(vec![]),
            ProgressState::new(
                UserProfile::new(),
                PerformanceTracker::new(),
                ProfileStorage::new(),
            ),
            ConfigState::default(),
        )
    }

    #[test]
    fn test_get_all_sort_modes() {
        let modes = SortMode::all();
        assert_eq!(modes.len(), 7);
        assert_eq!(modes[0], SortMode::ByDifficultyThenCategory);
    }

    #[test]
    fn test_show_sort_mode_selection_from_menu() {
        let (mut ui, mut game, mut progress, mut config) = create_test_context();
        let mut ctx = HandlerContext::new(&mut ui, &mut game, &mut progress, &mut config);

        let outcome = handle_show_sort_mode_selection(&mut ctx).unwrap();

        assert!(outcome.is_transition());
        if let HandlerOutcome::Transition(boxed) = outcome {
            if let TypedScreen::SortModeSelection(data) = *boxed {
                assert_eq!(data.return_to, ReturnDestination::Menu);
                assert!(data.selected_index < SortMode::all().len());
            } else {
                panic!("Expected SortModeSelection screen");
            }
        }
    }

    #[test]
    fn test_sort_mode_selection_up() {
        let (mut ui, mut game, mut progress, mut config) = create_test_context();
        let ctx = HandlerContext::new(&mut ui, &mut game, &mut progress, &mut config);

        let mut data = SortModeSelectionData {
            selected_index: 1,
            return_to: ReturnDestination::Menu,
        };

        let outcome = handle_sort_mode_selection_up(&mut data, &ctx).unwrap();
        assert!(outcome.is_stay());
        assert_eq!(data.selected_index, 0);
    }

    #[test]
    fn test_sort_mode_selection_up_wraparound() {
        let (mut ui, mut game, mut progress, mut config) = create_test_context();
        let ctx = HandlerContext::new(&mut ui, &mut game, &mut progress, &mut config);

        let mut data = SortModeSelectionData {
            selected_index: 0,
            return_to: ReturnDestination::Menu,
        };

        let outcome = handle_sort_mode_selection_up(&mut data, &ctx).unwrap();
        assert!(outcome.is_stay());
        let mode_count = SortMode::all().len();
        assert_eq!(data.selected_index, mode_count - 1);
    }

    #[test]
    fn test_sort_mode_selection_down() {
        let (mut ui, mut game, mut progress, mut config) = create_test_context();
        let ctx = HandlerContext::new(&mut ui, &mut game, &mut progress, &mut config);

        let mut data = SortModeSelectionData {
            selected_index: 0,
            return_to: ReturnDestination::Menu,
        };

        let outcome = handle_sort_mode_selection_down(&mut data, &ctx).unwrap();
        assert!(outcome.is_stay());
        assert_eq!(data.selected_index, 1);
    }

    #[test]
    fn test_sort_mode_selection_down_wraparound() {
        let (mut ui, mut game, mut progress, mut config) = create_test_context();
        let ctx = HandlerContext::new(&mut ui, &mut game, &mut progress, &mut config);

        let modes = SortMode::all();
        let mut data = SortModeSelectionData {
            selected_index: modes.len() - 1,
            return_to: ReturnDestination::Menu,
        };

        let outcome = handle_sort_mode_selection_down(&mut data, &ctx).unwrap();
        assert!(outcome.is_stay());
        assert_eq!(data.selected_index, 0);
    }

    #[test]
    fn test_sort_mode_selection_select() {
        let (mut ui, mut game, mut progress, mut config) = create_test_context();
        let mut ctx = HandlerContext::new(&mut ui, &mut game, &mut progress, &mut config);

        let data = SortModeSelectionData {
            selected_index: 1, // ByName
            return_to: ReturnDestination::Menu,
        };

        let outcome = handle_sort_mode_selection_select(&data, &mut ctx).unwrap();
        // Should stay in popup to allow selecting another mode or exiting with Esc
        assert!(outcome.is_stay());
        assert_eq!(ctx.config.sort_mode, SortMode::ByName);
    }
}
