use std::{
    sync::mpsc::{self, SendError},
    thread,
};

pub fn start_in_background() -> mpsc::Receiver<[f64; 4]> {
    let (color_tx, color_rx) = mpsc::sync_channel(0);

    thread::spawn(move || {
        let color = [0., 0., 0., 1.];

        println!("Color: {color:?}");

        loop {
            // The channel has no buffer, so this is synchronized to the frame
            // rate of the renderer.
            if let Err(SendError(_)) = color_tx.send(color) {
                // The other end has hung up. Time for us to shut down too.
                break;
            }
        }
    });

    color_rx
}
