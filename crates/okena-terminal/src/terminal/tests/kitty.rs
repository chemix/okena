//! Integration tests for the kitty keyboard protocol: the full chain from an
//! app pushing a mode (`CSI > flags u`) through alacritty's `TermMode` to
//! `Terminal::kitty_keyboard_flags()` and on into the key encoder. The
//! `input::tests` unit tests exercise the encoder with hand-built flags; these
//! tests guard the wiring those bypass — in particular that
//! `TermConfig::kitty_keyboard` stays enabled (without it alacritty silently
//! ignores every keyboard-mode sequence and the flag never flips).

use super::super::Terminal;
use super::super::types::TerminalSize;
use super::NullTransport;
use crate::input::{KeyEvent, KeyModifiers, key_to_bytes};
use std::sync::Arc;

fn term() -> Terminal {
    Terminal::new(
        "test-id".to_string(),
        TerminalSize::default(),
        Arc::new(NullTransport),
        "/tmp".to_string(),
    )
}

#[test]
fn push_and_pop_toggle_disambiguate_flag() {
    let t = term();
    assert!(
        !t.kitty_keyboard_flags().disambiguate_escape_codes,
        "disambiguate must start off"
    );

    // `CSI > 1 u` — push the disambiguate-escape-codes flag.
    t.process_output(b"\x1b[>1u");
    assert!(
        t.kitty_keyboard_flags().disambiguate_escape_codes,
        "push must set the flag (regression guard for TermConfig.kitty_keyboard)"
    );

    // `CSI < u` — pop one level off the stack, back to no modes.
    t.process_output(b"\x1b[<u");
    assert!(
        !t.kitty_keyboard_flags().disambiguate_escape_codes,
        "pop must clear the flag"
    );
}

#[test]
fn escape_encodes_as_csi_u_only_after_push() {
    let t = term();
    let esc = KeyEvent {
        key: "escape".to_string(),
        key_char: None,
        modifiers: KeyModifiers::default(),
    };

    // Before the app enables the protocol: legacy bare ESC.
    assert_eq!(
        key_to_bytes(&esc, false, t.kitty_keyboard_flags()),
        Some(b"\x1b".to_vec())
    );

    // After `CSI > 1 u`: the encoder reads the live flag and emits `CSI 27 u`.
    t.process_output(b"\x1b[>1u");
    assert_eq!(
        key_to_bytes(&esc, false, t.kitty_keyboard_flags()),
        Some(b"\x1b[27u".to_vec())
    );
}
