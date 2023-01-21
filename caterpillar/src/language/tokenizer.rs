use std::iter;

pub fn tokenize<'r>(
    chars: &'r mut impl Iterator<Item = char>,
    buf: &'r mut String,
) -> impl Iterator<Item = String> + 'r {
    iter::from_fn(|| {
        let mut state = State::NotStarted;

        for ch in chars.by_ref() {
            match state {
                State::NotStarted => {
                    if !ch.is_whitespace() {
                        state = State::ReadingToken;
                        buf.push(ch);
                    }
                }
                State::ReadingToken => {
                    if ch.is_whitespace() {
                        break;
                    } else {
                        buf.push(ch);
                    }
                }
            }
        }

        if buf.is_empty() {
            return None;
        }

        let token = buf.clone();
        buf.clear();

        Some(token)
    })
}

enum State {
    NotStarted,
    ReadingToken,
}
