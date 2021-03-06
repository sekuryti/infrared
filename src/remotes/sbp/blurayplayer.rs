use crate::protocols::sbp::SbpCommand;

use crate::remotecontrol::{Button, DeviceType, RemoteControl};

use Button::*;

pub struct SamsungBluRayPlayer;

impl RemoteControl for SamsungBluRayPlayer {
    const MODEL: &'static str = "Samsung BluRay Player";
    const DEVTYPE: DeviceType = DeviceType::BluRayPlayer;
    const ADDRESS: u32 = 32;
    type Cmd = SbpCommand;
    const BUTTONS: &'static [(u8, Button)] = &[
        (2, One),
        (3, Two),
        (4, Three),
        (5, Four),
        (6, Five),
        (7, Six),
        (8, Seven),
        (9, Eight),
        (10, Nine),
    ];
}
