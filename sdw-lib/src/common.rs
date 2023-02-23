type PosInt = u64;

#[derive(Debug, Clone, Copy, Default)]
pub struct PosInfo {
    pub line: PosInt,
    pub column: PosInt,
    pub length: PosInt,
}
