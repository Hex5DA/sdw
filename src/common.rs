type PosInt = u64;

#[derive(Debug)]
pub struct PosInfo {
    pub line: PosInt,
    pub column: PosInt,
    pub length: PosInt,
}
