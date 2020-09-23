pub(super) const NULL_REQ: [u8; 2] = [0, 0xFF];

#[derive(Copy, Clone)]
pub enum Command {
    Quit,
}
