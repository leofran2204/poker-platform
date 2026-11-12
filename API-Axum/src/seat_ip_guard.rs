//! Guarda antifraude de assento por rede (Termos de Uso, cláusula 2.3).
//!
//! Na entrada de uma mesa cash, a API recusa o jogador quando outra conta com
//! assento ativo na mesma mesa usa o mesmo IP público ou a mesma sub-rede
//! (/24 em IPv4, /64 em IPv6). Só impressões HMAC são gravadas; o endereço em
//! claro nunca vai ao banco (migration 061).
//!
//! Endereços que não identificam uma rede pública (loopback, rede privada,
//! link-local, não especificado) ficam fora da comparação: aparecem apenas em
//! laboratório, testes locais ou proxy mal configurado, e compará-los bloquearia
//! todos os jogadores atrás do mesmo gateway. `SEAT_IP_GUARD_EXEMPT_IPS`
//! (lista separada por vírgula) isenta endereços públicos usados em smoke
//! autorizado contra a demo.

use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::net::IpAddr;

type HmacSha256 = Hmac<Sha256>;

/// Impressões HMAC do endereço e da sub-rede de quem entra na mesa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatNetwork {
    pub ip_fingerprint: String,
    pub subnet_fingerprint: String,
}

/// Impressões do cliente, ou `None` quando o endereço é inválido, não público ou
/// isento por `SEAT_IP_GUARD_EXEMPT_IPS`. Sem impressão, o assento não é comparado.
pub fn seat_network(client_ip: &str) -> Option<SeatNetwork> {
    let exempt = std::env::var("SEAT_IP_GUARD_EXEMPT_IPS").unwrap_or_default();
    let pepper = std::env::var("KYC_DATA_PEPPER")
        .unwrap_or_else(|_| "development-kyc-data-pepper".to_string());
    seat_network_with(client_ip, &exempt, &pepper)
}

fn seat_network_with(client_ip: &str, exempt: &str, pepper: &str) -> Option<SeatNetwork> {
    let ip = canonical(client_ip.trim().parse().ok()?);
    if !is_public(ip) {
        return None;
    }
    let is_exempt = exempt
        .split(',')
        .filter_map(|entry| entry.trim().parse::<IpAddr>().ok())
        .any(|entry| canonical(entry) == ip);
    if is_exempt {
        return None;
    }
    let subnet = match ip {
        IpAddr::V4(v4) => {
            let octets = v4.octets();
            format!("{}.{}.{}.0/24", octets[0], octets[1], octets[2])
        }
        IpAddr::V6(v6) => {
            let segments = v6.segments();
            format!(
                "{:x}:{:x}:{:x}:{:x}::/64",
                segments[0], segments[1], segments[2], segments[3]
            )
        }
    };
    Some(SeatNetwork {
        ip_fingerprint: fingerprint(pepper, "ip", &ip.to_string()),
        subnet_fingerprint: fingerprint(pepper, "subnet", &subnet),
    })
}

/// IPv6 que embute IPv4 (`::ffff:a.b.c.d`) é tratado como o próprio IPv4.
fn canonical(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V6(v6) => v6
            .to_ipv4_mapped()
            .map(IpAddr::V4)
            .unwrap_or(IpAddr::V6(v6)),
        other => other,
    }
}

fn is_public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let octets = v4.octets();
            let shared_cgnat = octets[0] == 100 && (octets[1] & 0xc0) == 64;
            !(v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_broadcast()
                || shared_cgnat)
        }
        IpAddr::V6(v6) => {
            let first = v6.segments()[0];
            let unique_local = (first & 0xfe00) == 0xfc00;
            let link_local = (first & 0xffc0) == 0xfe80;
            !(v6.is_loopback() || v6.is_unspecified() || unique_local || link_local)
        }
    }
}

fn fingerprint(pepper: &str, kind: &str, value: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(pepper.as_bytes()).expect("HMAC accepts any key");
    mac.update(b"zero-tilt-seat-network-v1\0");
    mac.update(kind.as_bytes());
    mac.update(b"\0");
    mac.update(value.as_bytes());
    format!("{:x}", mac.finalize().into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PEPPER: &str = "test-pepper";

    fn network(ip: &str) -> Option<SeatNetwork> {
        seat_network_with(ip, "", PEPPER)
    }

    #[test]
    fn same_ipv4_and_same_slash_24_share_fingerprints() {
        let a = network("203.0.113.10").unwrap();
        let same = network("203.0.113.10").unwrap();
        let neighbour = network("203.0.113.200").unwrap();
        let other = network("198.51.100.10").unwrap();
        assert_eq!(a, same);
        assert_ne!(a.ip_fingerprint, neighbour.ip_fingerprint);
        assert_eq!(a.subnet_fingerprint, neighbour.subnet_fingerprint);
        assert_ne!(a.subnet_fingerprint, other.subnet_fingerprint);
    }

    #[test]
    fn ipv6_groups_by_slash_64_and_mapped_ipv4_is_canonical() {
        let a = network("2001:db8:1:2::10").unwrap();
        let b = network("2001:db8:1:2:ffff::1").unwrap();
        let c = network("2001:db8:1:3::10").unwrap();
        assert_eq!(a.subnet_fingerprint, b.subnet_fingerprint);
        assert_ne!(a.subnet_fingerprint, c.subnet_fingerprint);
        assert_eq!(network("::ffff:203.0.113.10"), network("203.0.113.10"));
    }

    #[test]
    fn non_public_and_invalid_addresses_are_not_compared() {
        for ip in [
            "127.0.0.1",
            "10.1.2.3",
            "172.16.0.9",
            "192.168.1.20",
            "169.254.1.1",
            "100.64.0.1",
            "0.0.0.0",
            "::1",
            "fd00::1",
            "fe80::1",
            "unknown-peer",
            "",
        ] {
            assert_eq!(network(ip), None, "{ip}");
        }
    }

    #[test]
    fn exempt_list_skips_only_listed_addresses() {
        let exempt = "198.51.100.7, 2001:db8::5";
        assert_eq!(seat_network_with("198.51.100.7", exempt, PEPPER), None);
        assert_eq!(seat_network_with("2001:db8::5", exempt, PEPPER), None);
        assert!(seat_network_with("198.51.100.8", exempt, PEPPER).is_some());
    }

    #[test]
    fn fingerprints_depend_on_the_secret_and_hide_the_address() {
        let a = seat_network_with("203.0.113.10", "", "one").unwrap();
        let b = seat_network_with("203.0.113.10", "", "two").unwrap();
        assert_ne!(a.ip_fingerprint, b.ip_fingerprint);
        assert_eq!(a.ip_fingerprint.len(), 64);
        assert!(a.ip_fingerprint.chars().all(|ch| ch.is_ascii_hexdigit()));
    }
}
