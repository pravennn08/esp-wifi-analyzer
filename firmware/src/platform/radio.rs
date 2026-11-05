//! Single-owner radio wrapper. Application models never expose driver types.
use esp_hal::peripherals::WIFI;
use esp_radio::wifi::WifiController;

pub struct Radio {
    _controller: WifiController<'static>,
}

impl Radio {
    pub fn new(peripheral: WIFI<'static>) -> Result<Self, esp_radio::wifi::WifiError> {
        let controller = WifiController::new(peripheral, Default::default())?;
        Ok(Self {
            _controller: controller,
        })
    }
}
