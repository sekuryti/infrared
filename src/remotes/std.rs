use crate::remotes::{
    nec::{SamsungTv, SpecialForMp3},
    rc5::Rc5CdPlayer,
    sbp::SamsungBluRayPlayer,
    DeviceType, RemoteControl, StandardButton,
};
use crate::ProtocolId;

pub fn remotes() -> Vec<RemoteControlData> {
    // Pretty much every remote ever manufactured :-)
    vec![
        RemoteControlData::new::<Rc5CdPlayer>(),
        RemoteControlData::new::<SamsungTv>(),
        RemoteControlData::new::<SpecialForMp3>(),
        RemoteControlData::new::<SamsungBluRayPlayer>(),
    ]
}

#[derive(Debug)]
pub struct RemoteControlData {
    pub model: &'static str,
    pub addr: u32,
    pub protocol: ProtocolId,
    pub dtype: DeviceType,
    pub mapping: &'static [(u8, StandardButton)],
}

impl RemoteControlData {
    pub fn new<R>() -> RemoteControlData
    where
        R: RemoteControl,
    {
        RemoteControlData {
            addr: R::ADDRESS,
            model: R::MODEL,
            dtype: R::DEVICE,
            protocol: R::PROTOCOL_ID,
            mapping: R::BUTTONS,
        }
    }
}
