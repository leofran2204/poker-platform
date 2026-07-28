use poker_engine::antifraud::{
    CollusionViolation, DeviceFingerprint, DeviceSecurityGuard, GeoLocation, PlayerSecurityContext,
};

#[test]
fn test_haversine_distance_calculation() {
    // Ponto A: Avenida Paulista, SP (-23.5615, -46.6559)
    let loc_a = GeoLocation::new(-23.5615, -46.6559);
    // Ponto B: A ~30 metros de distância
    let loc_b = GeoLocation::new(-23.5617, -46.6561);

    let dist = loc_a.distance_meters(&loc_b);
    assert!(
        dist > 10.0 && dist < 50.0,
        "Distância esperada ~30m, obtido: {:.2}m",
        dist
    );
}

#[test]
fn test_same_device_fingerprint_rejection() {
    let fp = DeviceFingerprint::new(
        "NVIDIA RTX 3080",
        "AudioSig123",
        "1920x1080",
        "FontHash99",
        "MacBookPro18,1",
        "macOS 14",
    );

    let p1 = PlayerSecurityContext {
        user_id: "Alice".into(),
        ip_address: "203.0.113.1".into(),
        device_fingerprint: fp.clone(),
        geo_location: None,
    };

    let p2 = PlayerSecurityContext {
        user_id: "Bob".into(),
        ip_address: "189.45.12.99".into(), // IP diferente!
        device_fingerprint: fp,
        geo_location: None,
    };

    let result = DeviceSecurityGuard::validate_table_seating_advanced(&[p1, p2]);
    assert!(matches!(
        result,
        Err(CollusionViolation::SameDeviceFingerprint(..))
    ));
}

#[test]
fn test_physical_proximity_guard_rejection() {
    let fp1 = DeviceFingerprint::new("GPU_1", "Audio_1", "1080p", "F1", "iPhone14", "iOS");
    let fp2 = DeviceFingerprint::new("GPU_2", "Audio_2", "4K", "F2", "GalaxyS23", "Android");

    let loc_home1 = GeoLocation::new(-23.5615, -46.6559);
    let loc_home2 = GeoLocation::new(-23.56155, -46.65595); // ~8 metros de distância!

    let p1 = PlayerSecurityContext {
        user_id: "Alice".into(),
        ip_address: "203.0.113.1".into(), // Wi-Fi de casa
        device_fingerprint: fp1,
        geo_location: Some(loc_home1),
    };

    let p2 = PlayerSecurityContext {
        user_id: "Bob".into(),
        ip_address: "177.92.14.200".into(), // 4G do celular!
        device_fingerprint: fp2,
        geo_location: Some(loc_home2),
    };

    let result = DeviceSecurityGuard::validate_table_seating_advanced(&[p1, p2]);
    assert!(matches!(
        result,
        Err(CollusionViolation::PhysicalProximityViolation(..))
    ));
}
