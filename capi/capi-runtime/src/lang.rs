pub fn lang(canvas_width: usize, canvas_height: usize, data: &mut [u8]) {
    set_all_pixels(canvas_width, canvas_height, data);
}

fn set_all_pixels(canvas_width: usize, canvas_height: usize, data: &mut [u8]) {
    let buffer_len = compute_draw_buffer_len(canvas_width, canvas_height);
    let mut i = 0;

    loop {
        if i >= buffer_len {
            break;
        }

        set_pixel(i, data);
        i = inc_pixel(i);
    }
}

fn compute_draw_buffer_len(canvas_width: usize, canvas_height: usize) -> usize {
    canvas_width * canvas_height * 4
}

fn set_pixel(i: usize, data: &mut [u8]) {
    set_red(i, data);
    set_green(i, data);
    set_blue(i, data);
    set_alpha(i, data);
}

fn set_red(i: usize, data: &mut [u8]) {
    data[i] = 0;
}

fn set_green(i: usize, data: &mut [u8]) {
    data[i + 1] = 255;
}

fn set_blue(i: usize, data: &mut [u8]) {
    data[i + 2] = 0;
}

fn set_alpha(i: usize, data: &mut [u8]) {
    data[i + 3] = 255;
}

fn inc_pixel(i: usize) -> usize {
    i + 4
}
