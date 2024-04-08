use pixels::{Pixels, SurfaceTexture};
use winit::{
    event::{Event, KeyEvent, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

use crate::capi::Program;

pub fn run(mut program: Program) -> anyhow::Result<()> {
    const TILES_PER_AXIS: usize = 32;
    const PIXELS_PER_TILE_AXIS: usize = 8;

    // I don't like the `as`, but I can't use `try_into` in a const context.
    // Given this is a screen resolution, this is unlikely to ever be a problem.
    const SIZE: usize = TILES_PER_AXIS * PIXELS_PER_TILE_AXIS;
    let size_u32: u32 =
        SIZE.try_into().expect("Expected `SIZE` to fit into `u32`");

    let mut tiles = [0; TILES_PER_AXIS * TILES_PER_AXIS];

    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Caterpillar")
        .build(&event_loop)?;

    let surface_texture = SurfaceTexture::new(size_u32, size_u32, &window);
    let mut pixels = Pixels::new(size_u32, size_u32, surface_texture)?;

    event_loop.run(|event, event_loop_window_target| match event {
        Event::AboutToWait => {
            program.run(TILES_PER_AXIS, TILES_PER_AXIS, &mut tiles);

            for tile_y in 0..TILES_PER_AXIS {
                for tile_x in 0..TILES_PER_AXIS {
                    let i = tile_y * TILES_PER_AXIS + tile_x;
                    let tile = tiles[i];

                    let color = if tile == 0 {
                        [0, 0, 0, 0]
                    } else {
                        [255, 255, 255, 255]
                    };

                    for offset_y in 0..PIXELS_PER_TILE_AXIS {
                        for offset_x in 0..PIXELS_PER_TILE_AXIS {
                            let num_channels = 4;

                            let frame_x = (tile_x * PIXELS_PER_TILE_AXIS
                                + offset_x)
                                * num_channels;
                            let frame_y = (tile_y * PIXELS_PER_TILE_AXIS
                                + offset_y)
                                * num_channels;

                            let i = frame_y * SIZE + frame_x;
                            pixels.frame_mut()[i..i + num_channels]
                                .copy_from_slice(&color);
                        }
                    }
                }
            }

            window.request_redraw();
        }
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            event_loop_window_target.exit();
        }
        Event::WindowEvent {
            event:
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                    ..
                },
            ..
        } => {
            event_loop_window_target.exit();
        }
        Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } => {
            if let Err(err) = pixels.render() {
                eprintln!("Render error: {err}");
            }
        }
        _ => {}
    })?;

    Ok(())
}
