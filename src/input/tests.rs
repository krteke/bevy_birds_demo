use super::*;

#[test]
fn scene_drag_resumes_after_crossing_the_panel() {
    let mut capture = DragCapture::default();
    let mut buttons = ButtonInput::default();
    buttons.press(MouseButton::Right);
    assert!(
        capture
            .scene_cursor(Some(Vec2::ZERO), false, &buttons, true)
            .is_some()
    );
    buttons.clear();
    assert!(
        capture
            .scene_cursor(Some(Vec2::ZERO), true, &buttons, true)
            .is_none()
    );
    assert!(
        capture
            .scene_cursor(Some(Vec2::ZERO), false, &buttons, true)
            .is_some()
    );
}

#[test]
fn drag_starting_in_ui_remains_captured_until_release() {
    let mut capture = DragCapture::default();
    let mut buttons = ButtonInput::default();
    buttons.press(MouseButton::Right);
    assert!(
        capture
            .scene_cursor(Some(Vec2::ZERO), true, &buttons, true)
            .is_none()
    );
    buttons.clear();
    assert!(
        capture
            .scene_cursor(Some(Vec2::ZERO), false, &buttons, true)
            .is_none()
    );
    buttons.release(MouseButton::Right);
    assert!(
        capture
            .scene_cursor(Some(Vec2::ZERO), false, &buttons, true)
            .is_some()
    );
}

#[test]
fn disabling_input_blocks_both_mouse_modes() {
    let mut capture = DragCapture::default();
    let mut buttons = ButtonInput::default();
    assert!(
        capture
            .scene_cursor(Some(Vec2::ZERO), false, &buttons, false)
            .is_none()
    );
    buttons.press(MouseButton::Right);
    assert!(
        capture
            .scene_cursor(Some(Vec2::ZERO), false, &buttons, false)
            .is_none()
    );
    assert!(
        capture
            .scene_cursor(Some(Vec2::ZERO), false, &buttons, true)
            .is_some()
    );
}

#[test]
fn left_ui_drag_is_captured_until_release() {
    let mut capture = DragCapture::default();
    let mut buttons = ButtonInput::default();
    buttons.press(MouseButton::Left);
    capture.scene_cursor(Some(Vec2::ZERO), true, &buttons, true);
    buttons.clear();
    assert!(
        capture
            .scene_cursor(Some(Vec2::ZERO), false, &buttons, true)
            .is_none()
    );
    buttons.release(MouseButton::Left);
    assert!(
        capture
            .scene_cursor(Some(Vec2::ZERO), false, &buttons, true)
            .is_some()
    );
}
