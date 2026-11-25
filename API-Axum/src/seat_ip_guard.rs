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

/// Recusa da inscrição no torneio: já há uma conta da mesma rede em cada mesa.
pub const SAME_NETWORK_TOURNAMENT_MESSAGE: &str = "Este torneio já tem, em todas as mesas, um jogador conectado pela mesma rede que você (mesmo IP ou sub-rede). Por regra antifraude, inscreva-se por outra conexão, como o 4G do celular, ou escolha outro torneio.";

/// Distribuição inicial do torneio separando contas da mesma rede (Termos 2.3).
///
/// `subnets[i]` é a impressão da sub-rede do i-ésimo inscrito, em ordem de
/// inscrição (`None` = sem impressão, nunca conflita); `capacities[t]` é quantos
/// jogadores a mesa `t` recebe. Devolve a mesa de cada inscrito e quantos
/// ficaram junto de alguém da mesma rede por falta de mesa.
///
/// Grupos maiores escolhem primeiro, cada membro numa mesa diferente, sempre as
/// mesas com mais vagas sobrando; pela construção de Gale–Ryser, isso separa
/// todos sempre que a separação é possível. O desempate (menos jogadores, depois
/// menor índice) reproduz o round-robin 0, 1, 2, 0, 1, 2... quando não há rede
/// repetida.
pub fn spread_by_network(subnets: &[Option<&str>], capacities: &[usize]) -> (Vec<usize>, usize) {
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut by_subnet: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for (player, &subnet) in subnets.iter().enumerate() {
        match subnet.and_then(|s| by_subnet.get(s).copied()) {
            Some(group) => groups[group].push(player),
            None => {
                if let Some(s) = subnet {
                    by_subnet.insert(s, groups.len());
                }
                groups.push(vec![player]);
            }
        }
    }
    // Ordem estável: maior grupo primeiro; empate pela inscrição mais antiga.
    groups.sort_by_key(|members| (std::cmp::Reverse(members.len()), members[0]));

    let mut assigned = vec![0usize; capacities.len()];
    let mut table_of = vec![0usize; subnets.len()];
    let mut shared = 0usize;
    for members in &groups {
        let mut used: Vec<usize> = Vec::with_capacity(members.len());
        for &player in members {
            let pick = |allow_used: bool| {
                (0..capacities.len())
                    .filter(|&t| assigned[t] < capacities[t])
                    .filter(|t| allow_used || !used.contains(t))
                    .min_by_key(|&t| {
                        (
                            std::cmp::Reverse(capacities[t] - assigned[t]),
                            assigned[t],
                            t,
                        )
                    })
            };
            let table = match pick(false) {
                Some(t) => t,
                None => {
                    shared += 1;
                    pick(true).unwrap_or(0)
                }
            };
            assigned[table] += 1;
            used.push(table);
            table_of[player] = table;
        }
    }
    (table_of, shared)
}

/// Escolha da mesa de quem volta pelo rebuy: a mesa com menos jogadores e assento
/// livre, evitando as que já têm alguém da mesma rede. Cada item é
/// `(jogadores, assento livre, tem alguém da mesma rede)`. Devolve o índice e se
/// a única saída foi uma mesa com a mesma rede.
pub fn pick_return_table(tables: &[(usize, Option<i16>, bool)]) -> Option<(usize, bool)> {
    let best = |avoid_shared: bool| {
        tables
            .iter()
            .enumerate()
            .filter(|(_, (_, free, shared))| free.is_some() && !(avoid_shared && *shared))
            .min_by_key(|(idx, (count, _, _))| (*count, *idx))
            .map(|(idx, (_, _, shared))| (idx, *shared))
    };
    best(true).or_else(|| best(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PEPPER: &str = "test-pepper";

    #[test]
    fn spread_without_repeated_networks_is_round_robin() {
        let subnets = [None, Some("a"), Some("b"), None, Some("c")];
        let (tables, shared) = spread_by_network(&subnets, &[2, 2, 1]);
        assert_eq!(tables, vec![0, 1, 2, 0, 1]);
        assert_eq!(shared, 0);
    }

    #[test]
    fn spread_separates_same_network_and_keeps_balance() {
        // Três contas da rede "casa" inscritas em sequência cairiam 0, 1, 2 no
        // round-robin; com "casa" em 0, 3 e 6 cairiam todas na mesa 0.
        let subnets = [
            Some("casa"),
            None,
            Some("x"),
            Some("casa"),
            None,
            Some("y"),
            Some("casa"),
            Some("z"),
            Some("w"),
        ];
        let (tables, shared) = spread_by_network(&subnets, &[3, 3, 3]);
        assert_eq!(shared, 0);
        let casa: Vec<usize> = [0, 3, 6].iter().map(|&p| tables[p]).collect();
        let mut distinct = casa.clone();
        distinct.sort_unstable();
        distinct.dedup();
        assert_eq!(distinct.len(), 3, "uma conta da rede por mesa: {casa:?}");
        for t in 0..3 {
            assert_eq!(tables.iter().filter(|&&x| x == t).count(), 3);
        }
    }

    #[test]
    fn spread_handles_two_groups_with_uneven_tables() {
        // 4 inscritos, mesas com 2/1/1 vagas: os dois pares ficam separados.
        let subnets = [Some("a"), Some("b"), Some("a"), Some("b")];
        let (tables, shared) = spread_by_network(&subnets, &[2, 1, 1]);
        assert_eq!(shared, 0);
        assert_ne!(tables[0], tables[2]);
        assert_ne!(tables[1], tables[3]);
        assert_eq!(tables.iter().filter(|&&t| t == 0).count(), 2);
    }

    #[test]
    fn spread_reports_when_separation_is_impossible() {
        // Quatro contas da mesma rede e três mesas: uma fica acompanhada.
        let subnets = [Some("a"); 4];
        let (tables, shared) = spread_by_network(&subnets, &[2, 1, 1]);
        assert_eq!(shared, 1);
        assert_eq!(tables.len(), 4);
    }

    #[test]
    fn return_table_avoids_same_network_when_possible() {
        // Mesa 0 é a mais vazia, mas tem alguém da mesma rede.
        let tables = [(3, Some(4), true), (5, Some(6), false), (4, None, false)];
        assert_eq!(pick_return_table(&tables), Some((1, false)));
        // Sem alternativa livre, volta para a mesa da mesma rede e avisa.
        let only_shared = [(3, Some(4), true), (5, None, false)];
        assert_eq!(pick_return_table(&only_shared), Some((0, true)));
        assert_eq!(pick_return_table(&[(9, None, false)]), None);
    }

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
