use crate::memory::Memory;

pub trait Storage {
    fn store(&self) -> Result<(), std::io::Error>;
    fn read(&self) -> Memory;
}
