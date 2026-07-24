use crate::antifraud::collusion::CollusionViolation;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Assinatura única de hardware do dispositivo (Device Fingerprint).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceFingerprint {
    pub hardware_hash: String,
    pub device_model: String,
    pub operating_system: String,
}

impl DeviceFingerprint {
    /// Gera um hash SHA-256 único combinando componentes de hardware (GPU WebGL, áudio Canvas API, tela e fontes).
    #[must_use]
    pub fn new(webgl_gpu: &str, canvas_audio_sig: &str, screen_res: &str, fonts_hash: &str, device_model: &str, os: &str) -> Self {
        let raw_components = format!("{}|{}|{}|{}|{}|{}", webgl_gpu, canvas_audio_sig, screen_res, fonts_hash, device_model, os);
        let hardware_hash = hex::encode(Sha256::digest(raw_components.as_bytes()));
        Self {
            hardware_hash,
            device_model: device_model.to_string(),
            operating_system: os.to_string(),
        }
    }
}

/// Coordenadas de geolocalização GPS do jogador.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeoLocation {
    pub latitude: f64,
    pub longitude: f64,
}

impl GeoLocation {
    #[must_use]
    pub const fn new(latitude: f64, longitude: f64) -> Self {
        Self { latitude, longitude }
    }

    /// Calcula a distância física exata em metros entre duas coordenadas de GPS usando a fórmula esférica de Haversine.
    #[must_use]
    pub fn distance_meters(&self, other: &Self) -> f64 {
        let earth_radius_meters = 6_371_000.0; // Raio da Terra em metros

        let lat1_rad = self.latitude.to_radians();
        let lat2_rad = other.latitude.to_radians();
        let delta_lat = (other.latitude - self.latitude).to_radians();
        let delta_lon = (other.longitude - self.longitude).to_radians();

        let a = (delta_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (delta_lon / 2.0).sin().powi(2);

        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

        earth_radius_meters * c
    }
}

/// Contexto de segurança completo de um jogador para validação de entrada na mesa.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSecurityContext {
    pub user_id: String,
    pub ip_address: String,
    pub device_fingerprint: DeviceFingerprint,
    pub geo_location: Option<GeoLocation>,
}

/// Validador de segurança avançado para prevenção de conluio (IP, Sub-rede /24, Device Fingerprint e Proximidade GPS).
pub struct DeviceSecurityGuard;

impl DeviceSecurityGuard {
    /// Valida o assento de um grupo de jogadores em uma mesa, aplicando a verificação de 4 camadas:
    /// 1. IP Estrito
    /// 2. Sub-rede `/24`
    /// 3. Hardware Device Fingerprint SHA-256
    /// 4. Proximidade Física GPS (< 50 metros)
    pub fn validate_table_seating_advanced(players: &[PlayerSecurityContext]) -> Result<(), CollusionViolation> {
        for i in 0..players.len() {
            for j in (i + 1)..players.len() {
                let p1 = &players[i];
                let p2 = &players[j];

                // 1. Trava de IP Estrito
                if p1.ip_address == p2.ip_address {
                    return Err(CollusionViolation::SameIpAddress(
                        p1.user_id.clone(),
                        p2.user_id.clone(),
                        p1.ip_address.clone(),
                    ));
                }

                // 2. Trava de Sub-rede `/24`
                if Self::extract_subnet_24(&p1.ip_address) == Self::extract_subnet_24(&p2.ip_address) {
                    return Err(CollusionViolation::SameSubnet(
                        p1.user_id.clone(),
                        p2.user_id.clone(),
                        Self::extract_subnet_24(&p1.ip_address),
                    ));
                }

                // 3. Trava de Device Fingerprint de Hardware SHA-256
                if p1.device_fingerprint.hardware_hash == p2.device_fingerprint.hardware_hash {
                    return Err(CollusionViolation::SameDeviceFingerprint(
                        p1.user_id.clone(),
                        p2.user_id.clone(),
                        p1.device_fingerprint.hardware_hash.clone(),
                    ));
                }

                // 4. Trava de Proximidade Física por Geolocalização GPS (< 50 metros)
                if let (Some(geo1), Some(geo2)) = (p1.geo_location, p2.geo_location) {
                    let distance = geo1.distance_meters(&geo2);
                    if distance < 50.0 {
                        return Err(CollusionViolation::PhysicalProximityViolation(
                            p1.user_id.clone(),
                            p2.user_id.clone(),
                            distance,
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    fn extract_subnet_24(ip: &str) -> String {
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() == 4 {
            format!("{}.{}.{}", parts[0], parts[1], parts[2])
        } else {
            ip.to_string()
        }
    }
}
