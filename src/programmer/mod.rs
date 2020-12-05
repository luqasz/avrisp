pub mod stk500v2;
use std::fmt;

#[allow(non_camel_case_types)]
pub enum Variant {
    STK500_V2,
    AVRISP_2,
}

impl fmt::Display for Variant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Variant::STK500_V2 => write!(f, "STK 500 v2"),
            Variant::AVRISP_2 => write!(f, "AVR ISP 2"),
        }
    }
}
