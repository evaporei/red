use sdl2::event::Event;
use sdl2::pixels::Color;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("red", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .build()
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    let mut quit = false;
    while !quit {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => quit = true,
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RED);
        canvas.clear();
        canvas.present();
    }

    Ok(())
}
