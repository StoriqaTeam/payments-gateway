use std::fmt::{self, Display};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    IOS,
    WEB,
    ANDROID,
}

impl Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DeviceType::IOS => f.write_str("IOS"),
            DeviceType::WEB => f.write_str("WEB"),
            DeviceType::ANDROID => f.write_str("ANDROID"),
        }
    }
}
