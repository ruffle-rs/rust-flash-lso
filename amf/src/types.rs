#[derive(Debug)]
pub struct Sol {
    pub header: SolHeader,
    pub body: Vec<SolElement>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SolHeader {
    pub version: [u8; 2],
    pub length: u32,
    pub signature: [u8; 10],
    pub name: String,
    //TODO: this could be an enum
    pub format_version: u8,
}

#[derive(Clone, Debug)]
pub struct SolElement {
    pub name: String,
    pub value: SolValue,
}

#[derive(Debug, Clone)]
pub enum SolValue {
    Number(f64),
    Bool(bool),
    String(String),
    Object(Vec<SolElement>),
    ObjectEnd,
    MixedArray(Vec<SolElement>),
    Null,
    Undefined,
    Reference(u16),
    // Array(Vec<SolElement>),
    Array(Vec<SolValue>),
    ValueArray(Vec<SolValue>),
    Date(f64, u16),
    LongString(String), // TODO: should this just be a string
    Unsupported,
    XML(String),
    TypedObject(String, Vec<SolElement>),
    // AMF3
    Integer(i32),
    ByteArray(Vec<u8>),
    //TODO: According to the spec these are both u32, hmm: might just be in relation to byte layout
    VectorInt(Vec<i32>, bool),
    VectorUInt(Vec<u32>, bool),
    //TODO: type
    VectorDouble(Vec<f64>, bool),
    VectorObject(Vec<SolValue>, String, bool),
    Dictionary(Vec<(SolValue, SolValue)>, bool),
}

#[derive(Clone, Debug)]
pub struct ClassDefinition {
    pub name: String,
    pub encoding: u8,
    pub attribute_count: u32,
    pub static_properties: Vec<String>,
    pub externalizable: bool,
}
