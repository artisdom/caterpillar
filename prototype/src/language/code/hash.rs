#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Hash {
    value: [u8; 32],
}
