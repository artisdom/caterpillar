use std::io::Stdout;

use crossterm::{
    cursor,
    style::{self, Stylize},
    QueueableCommand,
};

pub struct Area<'a> {
    out: &'a mut Stdout,
    offset: [u16; 2],
    size: [u16; 2],
    cursor: [u16; 2],
}

pub fn new(out: &mut Stdout, offset: [u16; 2], size: [u16; 2]) -> Area {
    Area {
        out,
        offset,
        size,
        cursor: [0; 2],
    }
}

pub fn size(area: &Area) -> [u16; 2] {
    area.size
}

pub fn new_line(area: &mut Area) {
    let [x, y] = &mut area.cursor;

    *x = 0;
    *y += 1;
}

pub fn write(area: &mut Area, s: &str) -> anyhow::Result<()> {
    let [ox, oy] = area.offset;
    let [x, y] = &mut area.cursor;

    area.out
        .queue(cursor::MoveTo(ox + *x, oy + *y))?
        .queue(style::PrintStyledContent(s.stylize()))?;

    let num_chars: u16 = s.chars().count().try_into().expect("String too long");
    *x += num_chars;

    Ok(())
}
