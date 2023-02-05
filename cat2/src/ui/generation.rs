use crate::cells;

use super::area::Area;

use super::area;

pub fn print_cells(
    area: &mut Area,
    cells: [bool; cells::NUM_CELLS],
) -> anyhow::Result<()> {
    for cell in cells {
        print_cell(area, cell)?;
    }

    area::move_to_new_line(area);

    Ok(())
}

fn print_cell(area: &mut Area, cell: bool) -> anyhow::Result<()> {
    let content = if cell { "#" } else { " " };
    area::draw(area, content);
    Ok(())
}
