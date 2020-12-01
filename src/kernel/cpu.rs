use crate::kernel::cpu_events::KeyboardStream;

pub trait CPUEvents {
    fn get_keyboard_stream(&self) -> KeyboardStream;
}

pub trait CPU {
    fn init(&self);
    fn hlt(&self);
}

impl<T> CPUEvents for T
where
    T: CPU,
{
    fn get_keyboard_stream(&self) -> KeyboardStream {
        return KeyboardStream::new();
    }
}
