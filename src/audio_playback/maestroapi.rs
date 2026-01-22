use crate::{gui::window::WasabiError, settings::MaestroSettings};

use maestroapi_rs::{FunctionType, MaestroAPI};

pub struct MaestroPlayer {
    maestro: MaestroAPI,
    port_data: bool,
}

impl MaestroPlayer {
    pub fn new(settings: &MaestroSettings) -> Result<Self, WasabiError> {
        let ports = if settings.use_ports {
            settings.num_ports
        } else {
            1
        };

        let maestro = MaestroAPI::initialize(ports, FunctionType::StandardRealtime)
            .map_err(|e| WasabiError::SynthError(format!("Failed to load Maestro: {e:#?}")))?;

        Ok(Self {
            maestro,
            port_data: settings.use_ports,
        })
    }

    pub fn reset(&mut self) {
        self.maestro.rt_reset_stream().unwrap_or_default();
    }

    pub fn push_events(&mut self, data: impl Iterator<Item = u32>) {
        let data: Vec<u32> = data.collect();
        self.maestro
            .rt_send_events(&data, self.port_data)
            .unwrap_or_default();
    }

    pub fn voice_count(&self) -> Option<u64> {
        if let Ok(stats) = self.maestro.get_statistics() {
            Some(stats.active_voice_count)
        } else {
            None
        }
    }
}
