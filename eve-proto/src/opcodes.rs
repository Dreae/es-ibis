pub enum EVEOpCode {
    None = 0x01,
    LongLong = 0x03,
    Long = 0x04,
    SignedShort = 0x05,
    Byte = 0x06,
    IntegerNegativeOne = 0x07,
    IntegerZero = 0x08,
    IntegerOne = 0x09,
    Real = 0x0a,
    RealZero = 0x0b,
    ShortString = 0x10,
    StringTableString = 0x11,
    WStringUCS2 = 0x12,
    LongString = 0x13,
    Tuple = 0x14,
    Dict = 0x16,
    Object = 0x17,
    EmptyTuple = 0x24,
    OneTuple = 0x25,
    SubStream = 0x2b,
    TwoTuple = 0x2c,
    WStringUTF8 = 0x2e,
    VarInteger = 0x2f
}

impl Into<u8> for EVEOpCode {
    fn into(self) -> u8 {
        self as u8
    }
}