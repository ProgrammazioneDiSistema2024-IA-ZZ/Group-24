use winit::event_loop::EventLoop;

pub fn get_screen_resolution() -> Option<(u32, u32)> {
    let event_loop = EventLoop::new();

    if let Some(primary_monitor) = event_loop.primary_monitor() {
        let size = primary_monitor.size();
        Some((size.width, size.height))
    } else {
        None
    }
}