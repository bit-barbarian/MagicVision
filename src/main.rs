mod cache;
mod capture;
mod constants;
mod messages;
mod recognition;
mod types;
use crossbeam::channel::RecvTimeoutError;
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
};
use magicvision::recognition::init::Engine;
use opencv::highgui;
use std::{io, time::Duration};

use crate::types::DynResult;

#[tokio::main]
async fn main() -> DynResult<()> {
    let engine = Engine::start().await?;

    highgui::named_window("MagicVision", highgui::WINDOW_NORMAL)?;
    highgui::named_window("WarpFrame", highgui::WINDOW_NORMAL)?;

    loop {
        match engine
            .recognition_rx
            .recv_timeout(Duration::from_millis(10))
        {
            Ok(result) => {
                highgui::imshow("MagicVision", &result.display_frame)?;
                if let Some(wf) = result.warped_frame {
                    highgui::imshow("WarpFrame", &wf)?;
                }
                if highgui::wait_key(1)? == 'q' as i32 {
                    break;
                }

                execute!(io::stdout(), Clear(ClearType::All))?;

                for (i, m) in result.matches.iter().enumerate() {
                    if let Some(card) = engine.cache.get(&m.card_id) {
                        println!("Match #{}: {} ({} {})", i, card.name, card.set, card.number);
                        println!("Distance:  {}", m.distance);
                    };
                }
            }
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        };
    }

    engine.stop()
}
