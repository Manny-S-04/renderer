#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Escape,
    Space,
    Enter,
    Up,
    Down,
    Left,
    Right,
    // lowercase a-z and 0-9
    Char(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Quit,
    KeyDown(Key),
    // left button only, window coordinates
    MouseDown(i32, i32),
    MouseUp(i32, i32),
    MouseMove(i32, i32),
}

pub fn is_quit(event: Event) -> bool {
    matches!(
        event,
        Event::Quit | Event::KeyDown(Key::Escape) | Event::KeyDown(Key::Char(b'q'))
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quit_keys() {
        assert!(is_quit(Event::Quit));
        assert!(is_quit(Event::KeyDown(Key::Escape)));
        assert!(is_quit(Event::KeyDown(Key::Char(b'q'))));
        assert!(!is_quit(Event::KeyDown(Key::Char(b'w'))));
        assert!(!is_quit(Event::MouseDown(0, 0)));
    }
}
