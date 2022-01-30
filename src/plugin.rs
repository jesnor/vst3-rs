pub trait Plugin {
    fn initialize(&self) -> bool { true }
    fn terminate(&self) -> bool { true }
}
