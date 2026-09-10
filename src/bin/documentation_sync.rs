//! Synchronizes operational status from the typed JSON contract.
//!
//! The JSON is the machine source. This binary generates:
//! - `Documentacao/STATUS_OPERACIONAL.md` (human-readable, entire file)
//! - a short pointer block between `DOCUMENTATION_SYNC` markers
//! - cash/MTT catalog tables in `DEMO_AMIGOS.md`
//!
//! Historical, legal, and educational prose remains authored by people.

use serde::Deserialize;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

const STATUS_FILE: &str = "Documentacao/STATUS_OPERACIONAL.json";
const STATUS_MARKDOWN: &str = "Documentacao/STATUS_OPERACIONAL.md";
const DEMO_FILE_NAME: &str = "DEMO_AMIGOS.md";

const START_MARKER: &str = "<!-- DOCUMENTATION_SYNC:START -->";
const END_MARKER: &str = "<!-- DOCUMENTATION_SYNC:END -->";
const CASH_START: &str = "<!-- DOCUMENTATION_SYNC:CASH_CATALOG:START -->";
const CASH_END: &str = "<!-- DOCUMENTATION_SYNC:CASH_CATALOG:END -->";
const MTT_START: &str = "<!-- DOCUMENTATION_SYNC:MTT_CATALOG:START -->";
const MTT_END: &str = "<!-- DOCUMENTATION_SYNC:MTT_CATALOG:END -->";

const ALLOWED_VARIANTS: &[&str] = &[
    "holdem",
    "short_deck",
    "short_deck_omaha",
    "ultimate_pineapple",
];

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct OperationalStatus {
    schema_version: u32,
    reviewed_on: String,
    cycle: Cycle,
    production: Production,
    stack: Stack,
    cash_tables: Vec<CashTable>,
    tournament: Tournament,
    wallets: Wallets,
    pix: Pix,
    presence: Presence,
    table_ownership: TableOwnership,
    flags: Flags,
    migrations_latest: u32,
    email_verification: bool,
    regulation_target: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cycle {
    id: String,
    title: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Production {
    certified: bool,
    environment: String,
    domain: String,
    host: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Stack {
    motor_api: String,
    frontend: String,
    frontend_legacy: String,
    edge: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct CashTable {
    name: String,
    variant: String,
    small_blind_cents: u64,
    big_blind_cents: u64,
    max_players: u8,
    buy_in_cents: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Tournament {
    start_local: String,
    timezone: String,
    auto_start_min_players: u8,
    tables_per_event: u8,
    fee_percent: u8,
    fee_split: FeeSplit,
    modes: Vec<String>,
    events: Vec<TournamentEvent>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct FeeSplit {
    l1: u8,
    l2: u8,
    house: u8,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TournamentEvent {
    name: String,
    variant: String,
    buy_in_cents: u64,
    gtd_cents: u64,
    table_max: u8,
    max_players: u16,
    reentries: u8,
    freeroll: bool,
    #[serde(default)]
    final_table_variant: Option<String>,
    #[serde(default)]
    final_table_max: Option<u8>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Wallets {
    pm_cash_cents: u64,
    pm_mtt_cents: u64,
    reset: String,
    timezone: String,
    rebuy: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pix {
    automatic_in_production: bool,
    vps_mock: bool,
    depix_sandbox_only: bool,
    manual_receiver: String,
    manual_key: String,
    withdraw_sla_hours: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Presence {
    public_get: bool,
    heartbeat_jwt: bool,
    ttl_seconds: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct TableOwnership {
    model: String,
    settlement: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Flags {
    mtt_live: bool,
    gameplay_ready: bool,
    visitor_shell: bool,
    hero_at_bottom: bool,
    showdown_sticky: bool,
    disconnect_not_leave: bool,
}

#[derive(Clone, Copy)]
enum Mode {
    Check,
    Write,
}

fn main() {
    let mode = match env::args().skip(1).collect::<Vec<_>>().as_slice() {
        [flag] if flag == "--check" => Mode::Check,
        [flag] if flag == "--write" => Mode::Write,
        _ => {
            eprintln!("Uso: cargo run --bin documentation-sync -- <--check|--write>");
            process::exit(2);
        }
    };

    if let Err(error) = synchronize(mode) {
        eprintln!("documentation-sync: {error}");
        process::exit(1);
    }
}

fn synchronize(mode: Mode) -> Result<(), String> {
    let root = repository_root()?;
    let status = read_status(&root)?;
    validate_status(&status)?;

    let mut stale = Vec::new();

    let status_md_path = root.join(STATUS_MARKDOWN);
    let rendered_md = render_status_md(&status);
    apply_file(mode, &status_md_path, &rendered_md, &root, &mut stale)?;

    let documents = markdown_documents(&root.join("Documentacao"))?;
    for path in &documents {
        let original = fs::read_to_string(path)
            .map_err(|error| format!("não foi possível ler {}: {error}", path.display()))?;
        let updated = synchronize_document(&original, &status, path)?;
        if updated != original {
            apply_contents(mode, path, &updated, &root, &mut stale)?;
        }
    }

    if stale.is_empty() {
        println!(
            "Documentação sincronizada: {} documentos Markdown verificados.",
            documents.len() + 1
        );
        Ok(())
    } else {
        Err(format!(
            "bloco de estado operacional ausente ou divergente em: {}. Rode `cargo run --bin documentation-sync -- --write` e versiona as alterações.",
            stale.join(", ")
        ))
    }
}

fn apply_file(
    mode: Mode,
    path: &Path,
    desired: &str,
    root: &Path,
    stale: &mut Vec<String>,
) -> Result<(), String> {
    let current = fs::read_to_string(path).unwrap_or_default();
    if current == desired {
        return Ok(());
    }
    apply_contents(mode, path, desired, root, stale)
}

fn apply_contents(
    mode: Mode,
    path: &Path,
    desired: &str,
    root: &Path,
    stale: &mut Vec<String>,
) -> Result<(), String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|error| format!("caminho documental fora do repositório: {error}"))?
        .display()
        .to_string();
    match mode {
        Mode::Check => stale.push(relative),
        Mode::Write => fs::write(path, desired)
            .map_err(|error| format!("não foi possível atualizar {relative}: {error}"))?,
    }
    Ok(())
}

fn markdown_documents(directory: &Path) -> Result<Vec<PathBuf>, String> {
    let mut documents = Vec::new();
    collect_markdown_documents(directory, &mut documents)?;
    documents.sort();

    if documents.is_empty() {
        return Err(format!(
            "nenhum documento Markdown encontrado em {}",
            directory.display()
        ));
    }

    Ok(documents)
}

fn collect_markdown_documents(
    directory: &Path,
    documents: &mut Vec<PathBuf>,
) -> Result<(), String> {
    for entry in fs::read_dir(directory)
        .map_err(|error| format!("não foi possível listar {}: {error}", directory.display()))?
    {
        let entry = entry.map_err(|error| format!("entrada documental inválida: {error}"))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("não foi possível inspecionar {}: {error}", path.display()))?;

        if file_type.is_dir() {
            if path
                .file_name()
                .is_some_and(|name| name == "historico" || name == "histórico")
            {
                continue;
            }
            collect_markdown_documents(&path, documents)?;
        } else if file_type.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
            && path
                .file_name()
                .is_some_and(|name| name != "STATUS_OPERACIONAL.md")
        {
            documents.push(path);
        }
    }

    Ok(())
}

fn repository_root() -> Result<PathBuf, String> {
    let current =
        env::current_dir().map_err(|error| format!("diretório atual inválido: {error}"))?;
    current
        .ancestors()
        .find(|candidate| {
            candidate.join("Cargo.toml").is_file() && candidate.join(STATUS_FILE).is_file()
        })
        .map(Path::to_path_buf)
        .ok_or_else(|| "execute o comando dentro do repositório Poker_Project".to_owned())
}

fn read_status(root: &Path) -> Result<OperationalStatus, String> {
    let path = root.join(STATUS_FILE);
    let content = fs::read_to_string(&path)
        .map_err(|error| format!("não foi possível ler {}: {error}", path.display()))?;
    parse_status(&content)
}

fn parse_status(content: &str) -> Result<OperationalStatus, String> {
    serde_json::from_str(content)
        .map_err(|error| format!("STATUS_OPERACIONAL.json inválido: {error}"))
}

fn validate_status(status: &OperationalStatus) -> Result<(), String> {
    if status.schema_version != 2 {
        return Err(format!(
            "schema_version {} não é suportado (esperado: 2)",
            status.schema_version
        ));
    }
    if !is_iso_date(&status.reviewed_on) {
        return Err("reviewed_on deve usar o formato YYYY-MM-DD".to_owned());
    }
    require_non_empty("cycle.id", &status.cycle.id)?;
    require_non_empty("cycle.title", &status.cycle.title)?;
    if status.production.certified {
        return Err(
            "production.certified deve ser false enquanto não houver gate explícito de certificação"
                .to_owned(),
        );
    }
    require_non_empty("production.environment", &status.production.environment)?;
    require_non_empty("production.domain", &status.production.domain)?;
    require_non_empty("production.host", &status.production.host)?;
    require_non_empty("stack.motor_api", &status.stack.motor_api)?;
    require_non_empty("stack.frontend", &status.stack.frontend)?;
    require_non_empty("stack.frontend_legacy", &status.stack.frontend_legacy)?;
    require_non_empty("stack.edge", &status.stack.edge)?;

    if status.cash_tables.is_empty() {
        return Err("cash_tables não pode ser vazio".to_owned());
    }
    for (index, table) in status.cash_tables.iter().enumerate() {
        require_non_empty(&format!("cash_tables[{index}].name"), &table.name)?;
        validate_variant(&table.variant)?;
        if table.small_blind_cents == 0 || table.big_blind_cents == 0 || table.buy_in_cents == 0 {
            return Err(format!(
                "cash_tables[{index}]: blinds e buy_in_cents devem ser > 0"
            ));
        }
        if !(2..=10).contains(&table.max_players) {
            return Err(format!(
                "cash_tables[{index}].max_players deve estar em 2..=10"
            ));
        }
    }

    validate_hh_mm(&status.tournament.start_local)?;
    require_non_empty("tournament.timezone", &status.tournament.timezone)?;
    if status.tournament.auto_start_min_players < 2 {
        return Err("tournament.auto_start_min_players deve ser >= 2".to_owned());
    }
    if status.tournament.tables_per_event == 0 {
        return Err("tournament.tables_per_event deve ser >= 1".to_owned());
    }
    if status.tournament.fee_percent > 100 {
        return Err("tournament.fee_percent deve estar em 0..=100".to_owned());
    }
    let split = &status.tournament.fee_split;
    if u16::from(split.l1) + u16::from(split.l2) + u16::from(split.house) != 100 {
        return Err("tournament.fee_split deve somar 100".to_owned());
    }
    if status.tournament.modes.is_empty() {
        return Err("tournament.modes não pode ser vazio".to_owned());
    }
    for mode in &status.tournament.modes {
        if mode != "play" && mode != "real" {
            return Err(format!("tournament.modes contém valor inválido: {mode}"));
        }
    }
    if status.tournament.events.is_empty() {
        return Err("tournament.events não pode ser vazio".to_owned());
    }
    for (index, event) in status.tournament.events.iter().enumerate() {
        require_non_empty(&format!("tournament.events[{index}].name"), &event.name)?;
        validate_variant(&event.variant)?;
        if event.freeroll {
            if event.buy_in_cents != 0 {
                return Err(format!(
                    "tournament.events[{index}]: freeroll deve ter buy_in_cents = 0"
                ));
            }
        } else if event.buy_in_cents == 0 {
            return Err(format!(
                "tournament.events[{index}]: evento pago deve ter buy_in_cents > 0"
            ));
        }
        if !(2..=10).contains(&event.table_max) {
            return Err(format!(
                "tournament.events[{index}].table_max deve estar em 2..=10"
            ));
        }
        if event.max_players < u16::from(event.table_max) {
            return Err(format!(
                "tournament.events[{index}].max_players deve ser >= table_max"
            ));
        }
        if let Some(variant) = &event.final_table_variant {
            validate_variant(variant)?;
        }
        if let Some(max) = event.final_table_max {
            if !(2..=10).contains(&max) {
                return Err(format!(
                    "tournament.events[{index}].final_table_max deve estar em 2..=10"
                ));
            }
        }
    }

    if status.wallets.pm_cash_cents == 0 || status.wallets.pm_mtt_cents == 0 {
        return Err("wallets PM devem ter centavos > 0".to_owned());
    }
    if status.wallets.reset != "daily" {
        return Err("wallets.reset deve ser \"daily\"".to_owned());
    }
    require_non_empty("wallets.timezone", &status.wallets.timezone)?;

    if status.pix.automatic_in_production {
        return Err(
            "pix.automatic_in_production deve ser false enquanto o código rejeitar PIX em production"
                .to_owned(),
        );
    }
    require_non_empty("pix.manual_receiver", &status.pix.manual_receiver)?;
    require_non_empty("pix.manual_key", &status.pix.manual_key)?;
    if status.pix.withdraw_sla_hours == 0 {
        return Err("pix.withdraw_sla_hours deve ser > 0".to_owned());
    }
    if status.presence.ttl_seconds == 0 {
        return Err("presence.ttl_seconds deve ser > 0".to_owned());
    }
    require_non_empty("table_ownership.model", &status.table_ownership.model)?;
    require_non_empty(
        "table_ownership.settlement",
        &status.table_ownership.settlement,
    )?;
    if status.migrations_latest == 0 {
        return Err("migrations_latest deve ser >= 1".to_owned());
    }
    if !is_year_month(&status.regulation_target) {
        return Err("regulation_target deve usar o formato YYYY-MM".to_owned());
    }

    Ok(())
}

fn validate_variant(variant: &str) -> Result<(), String> {
    if ALLOWED_VARIANTS.contains(&variant) {
        Ok(())
    } else {
        Err(format!("variante inválida: {variant}"))
    }
}

fn require_non_empty(name: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("campo obrigatório vazio: {name}"))
    } else {
        Ok(())
    }
}

fn is_iso_date(value: &str) -> bool {
    value.len() == 10
        && value.chars().enumerate().all(|(index, character)| {
            matches!(index, 4 | 7) && character == '-'
                || !matches!(index, 4 | 7) && character.is_ascii_digit()
        })
}

fn is_year_month(value: &str) -> bool {
    value.len() == 7
        && value.chars().enumerate().all(|(index, character)| {
            index == 4 && character == '-' || index != 4 && character.is_ascii_digit()
        })
}

fn validate_hh_mm(value: &str) -> Result<(), String> {
    let bytes = value.as_bytes();
    if bytes.len() != 5 || bytes[2] != b':' {
        return Err("tournament.start_local deve usar o formato HH:MM".to_owned());
    }
    let hours: u8 = value[..2]
        .parse()
        .map_err(|_| "tournament.start_local inválido".to_owned())?;
    let minutes: u8 = value[3..]
        .parse()
        .map_err(|_| "tournament.start_local inválido".to_owned())?;
    if hours > 23 || minutes > 59 {
        return Err("tournament.start_local fora do intervalo".to_owned());
    }
    Ok(())
}

fn synchronize_document(
    original: &str,
    status: &OperationalStatus,
    path: &Path,
) -> Result<String, String> {
    let newline = if original.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let pointer = render_pointer_block(status, newline);
    let mut updated =
        replace_or_append_block(original, START_MARKER, END_MARKER, &pointer, newline)?;

    if path.file_name().is_some_and(|name| name == DEMO_FILE_NAME) {
        let cash = wrap_block(
            CASH_START,
            CASH_END,
            &render_cash_table(status, newline),
            newline,
        );
        updated = replace_existing_block(&updated, CASH_START, CASH_END, &cash)?;
        let mtt = wrap_block(
            MTT_START,
            MTT_END,
            &render_mtt_table(status, newline),
            newline,
        );
        updated = replace_existing_block(&updated, MTT_START, MTT_END, &mtt)?;
    }

    Ok(updated)
}

fn replace_or_append_block(
    original: &str,
    start_marker: &str,
    end_marker: &str,
    block: &str,
    newline: &str,
) -> Result<String, String> {
    let start_positions = marker_positions(original, start_marker);
    let end_positions = marker_positions(original, end_marker);

    match (start_positions.as_slice(), end_positions.as_slice()) {
        ([], []) => {
            let separator = if original.is_empty() || original.ends_with('\n') {
                ""
            } else {
                newline
            };
            Ok(format!("{original}{separator}{newline}{block}{newline}"))
        }
        ([start], [end]) if *start < *end => {
            let end_of_marker = *end + end_marker.len();
            Ok(format!(
                "{}{}{}",
                &original[..*start],
                block,
                &original[end_of_marker..]
            ))
        }
        _ => Err(format!(
            "marcadores {start_marker} ausentes, duplicados ou fora de ordem"
        )),
    }
}

fn replace_existing_block(
    original: &str,
    start_marker: &str,
    end_marker: &str,
    block: &str,
) -> Result<String, String> {
    let start_positions = marker_positions(original, start_marker);
    let end_positions = marker_positions(original, end_marker);
    match (start_positions.as_slice(), end_positions.as_slice()) {
        ([start], [end]) if *start < *end => {
            let end_of_marker = *end + end_marker.len();
            Ok(format!(
                "{}{}{}",
                &original[..*start],
                block,
                &original[end_of_marker..]
            ))
        }
        _ => Err(format!(
            "marcadores {start_marker} obrigatórios ausentes, duplicados ou fora de ordem"
        )),
    }
}

fn marker_positions(content: &str, marker: &str) -> Vec<usize> {
    content
        .match_indices(marker)
        .map(|(index, _)| index)
        .collect()
}

fn wrap_block(start: &str, end: &str, inner: &str, newline: &str) -> String {
    format!("{start}{newline}{inner}{newline}{end}")
}

fn render_pointer_block(status: &OperationalStatus, newline: &str) -> String {
    [
        START_MARKER.to_owned(),
        format!(
            "> **{}** ({}) — demo `{}` · sem certificação de produção · PIX automático desligado.",
            status.cycle.id, status.reviewed_on, status.production.domain
        ),
        "> Fatos (catálogo, carteiras, limites): [`STATUS_OPERACIONAL.md`](STATUS_OPERACIONAL.md)."
            .to_owned(),
        END_MARKER.to_owned(),
    ]
    .join(newline)
}

fn render_status_md(status: &OperationalStatus) -> String {
    let n = "\n";
    let mut out = String::new();
    out.push_str("<!-- generated by documentation-sync; do not edit -->");
    out.push_str(n);
    out.push_str(n);
    out.push_str(&format!("# Estado operacional — {}{}", status.cycle.id, n));
    out.push_str(n);
    out.push_str(&format!(
        "**{}** — {}. Revisado em **{}**. Ambiente: **{}** em [{}](https://{}) ({}).{}",
        status.cycle.id,
        status.cycle.title,
        status.reviewed_on,
        status.production.environment,
        status.production.domain,
        status.production.domain,
        status.production.host,
        n
    ));
    out.push_str(n);
    out.push_str(&format!(
        "**Limites:** sem certificação de produção · PIX automático desligado · mesas com dono **{}** (settlement {}).{}",
        ownership_label(&status.table_ownership.model),
        status.table_ownership.settlement,
        n
    ));
    out.push_str(n);
    out.push_str(&format!(
        "Fonte máquina: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.{}{}",
        n, n
    ));

    out.push_str(&format!("## Stack{n}{n}"));
    out.push_str(&format!("- Motor/API: {}{n}", status.stack.motor_api));
    out.push_str(&format!("- Frontend: {}{n}", status.stack.frontend));
    out.push_str(&format!("- Legado: {}{n}", status.stack.frontend_legacy));
    out.push_str(&format!("- Edge: {}{n}{n}", status.stack.edge));

    out.push_str(&format!("## Cash (Play Money e Jogo Real){n}{n}"));
    out.push_str(&render_cash_table(status, n));
    out.push_str(n);
    out.push_str(n);
    out.push_str(&format!(
        "Frentes fixas (`min_buy_in = max_buy_in`). Migrations até **{:03}**.{}{}",
        status.migrations_latest, n, n
    ));

    let t = &status.tournament;
    out.push_str(&format!("## Torneios{n}{n}"));
    out.push_str(&format!(
        "Agenda **{}** `{}`, auto-start com **{}+**, **{}** mesas por evento, taxa **{}%** por cima (freeroll sem taxa; split {}/{}/{}). Modos: {}.{}{}",
        t.start_local,
        t.timezone,
        t.auto_start_min_players,
        t.tables_per_event,
        t.fee_percent,
        t.fee_split.l1,
        t.fee_split.l2,
        t.fee_split.house,
        t.modes.join(" + "),
        n,
        n
    ));
    out.push_str(&render_mtt_table(status, n));
    out.push_str(n);
    out.push_str(n);

    out.push_str(&format!("## Carteiras{n}{n}"));
    out.push_str(&format!(
        "| Item | Play Money | Jogo Real |{n}|------|------------|-----------|{n}"
    ));
    out.push_str(&format!(
        "| Cash | {} / {} ({}, sem rebuy) | Depósito manual PIX + aprovação |{n}",
        format_reais(status.wallets.pm_cash_cents),
        reset_label(&status.wallets.reset),
        status.wallets.timezone
    ));
    out.push_str(&format!(
        "| Torneio | {} / {} | Buy-in com saldo real |{n}",
        format_reais(status.wallets.pm_mtt_cents),
        reset_label(&status.wallets.reset)
    ));
    out.push_str(&format!(
        "| Mistura | **Não** — PM não entra em mesa Real | idem |{n}{n}"
    ));
    if status.wallets.rebuy {
        out.push_str(&format!("Rebuy de carteira PM: sim.{n}{n}"));
    } else {
        out.push_str(&format!(
            "Zerou a carteira PM: espera o reset. Entradas/rebuys de torneio ilimitados **com saldo**.{}{}",
            n, n
        ));
    }

    out.push_str(&format!("## PIX{n}{n}"));
    out.push_str(&format!("- Automático em production: **não**{n}"));
    out.push_str(&format!(
        "- VPS: {} · DePix: {}{n}",
        if status.pix.vps_mock {
            "mock"
        } else {
            "não-mock"
        },
        if status.pix.depix_sandbox_only {
            "somente sandbox não produtivo"
        } else {
            "fora do sandbox"
        }
    ));
    out.push_str(&format!(
        "- Depósito manual: recebedor **{}**, chave `{}`{n}",
        status.pix.manual_receiver, status.pix.manual_key
    ));
    out.push_str(&format!(
        "- Saque: informar chave própria; recebimento em até **{}h**{}{}",
        status.pix.withdraw_sla_hours, n, n
    ));

    out.push_str(&format!("## Presença{n}{n}"));
    out.push_str(&format!(
        "- GET `/api/presence/online` público: {}{n}",
        yes_no(status.presence.public_get)
    ));
    out.push_str(&format!(
        "- Heartbeat JWT: {}{n}",
        yes_no(status.presence.heartbeat_jwt)
    ));
    out.push_str(&format!(
        "- TTL: **{}s**{}{}",
        status.presence.ttl_seconds, n, n
    ));

    out.push_str(&format!("## Flags do ciclo{n}{n}"));
    out.push_str(&format!(
        "- MTT ao vivo (mesmo protocolo WS do cash): {}{n}",
        yes_no(status.flags.mtt_live)
    ));
    out.push_str(&format!(
        "- `gameplay_ready`: {}{n}",
        yes_no(status.flags.gameplay_ready)
    ));
    out.push_str(&format!(
        "- Home de vitrine para visitante: {}{n}",
        yes_no(status.flags.visitor_shell)
    ));
    out.push_str(&format!(
        "- Herói sempre embaixo: {}{n}",
        yes_no(status.flags.hero_at_bottom)
    ));
    out.push_str(&format!(
        "- Painel de showdown fica até fechar: {}{n}",
        yes_no(status.flags.showdown_sticky)
    ));
    out.push_str(&format!(
        "- Drop de WS = Disconnect, não Leave: {}{}{}",
        yes_no(status.flags.disconnect_not_leave),
        n,
        n
    ));

    out.push_str(&format!("## Conta e regulação{n}{n}"));
    out.push_str(&format!(
        "- Verificação de e-mail obrigatória: {}{n}",
        yes_no(status.email_verification)
    ));
    out.push_str(&format!(
        "- Trilho de regulação/KYC: **{}**{}{}",
        status.regulation_target, n, n
    ));

    out.push_str(&format!("## Onde ler o resto{n}{n}"));
    out.push_str(&format!(
        "- Convite e jornada: [`DEMO_AMIGOS.md`](DEMO_AMIGOS.md){n}"
    ));
    out.push_str(&format!(
        "- Painel tático: [`DASHBOARD.md`](DASHBOARD.md){n}"
    ));
    out.push_str(&format!(
        "- Histórico: [`DEVELOPMENT_LOG.md`](DEVELOPMENT_LOG.md){n}"
    ));
    out.push_str(&format!("- Qualidade: [`QUALITY.md`](QUALITY.md){n}"));

    out
}

fn render_cash_table(status: &OperationalStatus, newline: &str) -> String {
    let mut rows = vec![
        "| Mesa | Jogo | Blinds | Cap | Frente |".to_owned(),
        "|------|------|--------|-----|--------|".to_owned(),
    ];
    for table in &status.cash_tables {
        rows.push(format!(
            "| {} | {} | {} | {} | {} |",
            cash_short_label(table),
            variant_label(&table.variant),
            format_blinds(table.small_blind_cents, table.big_blind_cents),
            table.max_players,
            format_reais(table.buy_in_cents)
        ));
    }
    rows.join(newline)
}

fn render_mtt_table(status: &OperationalStatus, newline: &str) -> String {
    let mut rows = vec![
        "| Evento | Variante | Buy-in | GTD | Cap | Máx. | Reentradas |".to_owned(),
        "|--------|----------|--------|-----|-----|------|------------|".to_owned(),
    ];
    for event in &status.tournament.events {
        let buy_in = if event.freeroll {
            "Grátis".to_owned()
        } else {
            format_reais(event.buy_in_cents)
        };
        let gtd = if event.gtd_cents == 0 {
            "—".to_owned()
        } else {
            format_reais(event.gtd_cents)
        };
        let mut name = event.name.clone();
        if let (Some(variant), Some(max)) = (&event.final_table_variant, event.final_table_max) {
            name.push_str(&format!(" (FT {} {max}-max)", variant_label(variant)));
        }
        rows.push(format!(
            "| {} | {} | {} | {} | {} | {} | {} |",
            name,
            variant_label(&event.variant),
            buy_in,
            gtd,
            event.table_max,
            event.max_players,
            event.reentries
        ));
    }
    rows.join(newline)
}

fn cash_short_label(table: &CashTable) -> String {
    let blinds = if table.small_blind_cents == table.big_blind_cents {
        format_stake(table.small_blind_cents)
    } else {
        format!(
            "{}/{}",
            format_stake(table.small_blind_cents),
            format_stake(table.big_blind_cents)
        )
    };
    match table.variant.as_str() {
        "holdem" => format!("NL {blinds}"),
        "short_deck" => format!("SD {blinds}"),
        "short_deck_omaha" => format!("SD Omaha {blinds}"),
        "ultimate_pineapple" => format!("Pineapple {blinds}"),
        other => other.to_owned(),
    }
}

fn variant_label(variant: &str) -> &'static str {
    match variant {
        "holdem" => "Texas Hold’em",
        "short_deck" => "Texas Short Deck",
        "short_deck_omaha" => "Short Deck Omaha",
        "ultimate_pineapple" => "Ultimate Pineapple",
        _ => "desconhecida",
    }
}

fn format_blinds(small: u64, big: u64) -> String {
    format!("{} / {}", format_stake(small), format_stake(big))
}

fn format_stake(cents: u64) -> String {
    let whole = cents / 100;
    let frac = cents % 100;
    if whole == 0 {
        format!("0,{frac:02}")
    } else if frac == 0 {
        format!("{whole}")
    } else {
        format!("{whole},{frac:02}")
    }
}

fn format_reais(cents: u64) -> String {
    let whole = cents / 100;
    let frac = cents % 100;
    if frac == 0 {
        format!("R$ {whole}")
    } else {
        format!("R$ {whole},{frac:02}")
    }
}

fn reset_label(reset: &str) -> &'static str {
    match reset {
        "daily" => "dia",
        _ => "reset",
    }
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "sim"
    } else {
        "não"
    }
}

fn ownership_label(model: &str) -> &str {
    match model {
        "single_process" => "único por processo",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_status() -> OperationalStatus {
        OperationalStatus {
            schema_version: 2,
            reviewed_on: "2026-09-10".to_owned(),
            cycle: Cycle {
                id: "S24".to_owned(),
                title: "ciclo de teste".to_owned(),
            },
            production: Production {
                certified: false,
                environment: "demo/staging".to_owned(),
                domain: "zerotiltpoker.net".to_owned(),
                host: "VPS Hostinger".to_owned(),
            },
            stack: Stack {
                motor_api: "Rust".to_owned(),
                frontend: "Frontend-Web".to_owned(),
                frontend_legacy: "Dioxus removido".to_owned(),
                edge: "Caddy".to_owned(),
            },
            cash_tables: vec![
                CashTable {
                    name: "NL 0,25".to_owned(),
                    variant: "holdem".to_owned(),
                    small_blind_cents: 25,
                    big_blind_cents: 25,
                    max_players: 9,
                    buy_in_cents: 2500,
                },
                CashTable {
                    name: "NL 0,75/1,50".to_owned(),
                    variant: "holdem".to_owned(),
                    small_blind_cents: 75,
                    big_blind_cents: 150,
                    max_players: 9,
                    buy_in_cents: 15000,
                },
                CashTable {
                    name: "SD".to_owned(),
                    variant: "short_deck".to_owned(),
                    small_blind_cents: 25,
                    big_blind_cents: 50,
                    max_players: 8,
                    buy_in_cents: 7500,
                },
                CashTable {
                    name: "Omaha".to_owned(),
                    variant: "short_deck_omaha".to_owned(),
                    small_blind_cents: 50,
                    big_blind_cents: 50,
                    max_players: 5,
                    buy_in_cents: 10000,
                },
                CashTable {
                    name: "Pineapple".to_owned(),
                    variant: "ultimate_pineapple".to_owned(),
                    small_blind_cents: 50,
                    big_blind_cents: 50,
                    max_players: 6,
                    buy_in_cents: 7500,
                },
            ],
            tournament: Tournament {
                start_local: "21:30".to_owned(),
                timezone: "America/Sao_Paulo".to_owned(),
                auto_start_min_players: 5,
                tables_per_event: 3,
                fee_percent: 15,
                fee_split: FeeSplit {
                    l1: 18,
                    l2: 12,
                    house: 70,
                },
                modes: vec!["play".to_owned(), "real".to_owned()],
                events: vec![TournamentEvent {
                    name: "Texas Hold’em".to_owned(),
                    variant: "holdem".to_owned(),
                    buy_in_cents: 1500,
                    gtd_cents: 15000,
                    table_max: 9,
                    max_players: 27,
                    reentries: 1,
                    freeroll: false,
                    final_table_variant: None,
                    final_table_max: None,
                }],
            },
            wallets: Wallets {
                pm_cash_cents: 15000,
                pm_mtt_cents: 15000,
                reset: "daily".to_owned(),
                timezone: "America/Sao_Paulo".to_owned(),
                rebuy: false,
            },
            pix: Pix {
                automatic_in_production: false,
                vps_mock: true,
                depix_sandbox_only: true,
                manual_receiver: "Leofran".to_owned(),
                manual_key: "6eefcd53-686e-42d4-a062-03751336251c".to_owned(),
                withdraw_sla_hours: 24,
            },
            presence: Presence {
                public_get: true,
                heartbeat_jwt: true,
                ttl_seconds: 90,
            },
            table_ownership: TableOwnership {
                model: "single_process".to_owned(),
                settlement: "HMAC".to_owned(),
            },
            flags: Flags {
                mtt_live: true,
                gameplay_ready: true,
                visitor_shell: true,
                hero_at_bottom: true,
                showdown_sticky: true,
                disconnect_not_leave: true,
            },
            migrations_latest: 53,
            email_verification: true,
            regulation_target: "2027-01".to_owned(),
        }
    }

    fn sample_json() -> String {
        serde_json::to_string(&serde_json::json!({
            "schema_version": 2,
            "reviewed_on": "2026-09-10",
            "cycle": { "id": "S24", "title": "ciclo de teste" },
            "production": {
                "certified": false,
                "environment": "demo/staging",
                "domain": "zerotiltpoker.net",
                "host": "VPS Hostinger"
            },
            "stack": {
                "motor_api": "Rust",
                "frontend": "Frontend-Web",
                "frontend_legacy": "Dioxus removido",
                "edge": "Caddy"
            },
            "cash_tables": [{
                "name": "NL",
                "variant": "holdem",
                "small_blind_cents": 25,
                "big_blind_cents": 25,
                "max_players": 9,
                "buy_in_cents": 2500
            }],
            "tournament": {
                "start_local": "21:30",
                "timezone": "America/Sao_Paulo",
                "auto_start_min_players": 5,
                "tables_per_event": 3,
                "fee_percent": 15,
                "fee_split": { "l1": 18, "l2": 12, "house": 70 },
                "modes": ["play", "real"],
                "events": [{
                    "name": "Texas",
                    "variant": "holdem",
                    "buy_in_cents": 1500,
                    "gtd_cents": 15000,
                    "table_max": 9,
                    "max_players": 27,
                    "reentries": 1,
                    "freeroll": false
                }]
            },
            "wallets": {
                "pm_cash_cents": 15000,
                "pm_mtt_cents": 15000,
                "reset": "daily",
                "timezone": "America/Sao_Paulo",
                "rebuy": false
            },
            "pix": {
                "automatic_in_production": false,
                "vps_mock": true,
                "depix_sandbox_only": true,
                "manual_receiver": "Leofran",
                "manual_key": "abc",
                "withdraw_sla_hours": 24
            },
            "presence": {
                "public_get": true,
                "heartbeat_jwt": true,
                "ttl_seconds": 90
            },
            "table_ownership": { "model": "single_process", "settlement": "HMAC" },
            "flags": {
                "mtt_live": true,
                "gameplay_ready": true,
                "visitor_shell": true,
                "hero_at_bottom": true,
                "showdown_sticky": true,
                "disconnect_not_leave": true
            },
            "migrations_latest": 53,
            "email_verification": true,
            "regulation_target": "2027-01"
        }))
        .unwrap()
    }

    #[test]
    fn inserts_then_replaces_only_the_generated_block() {
        let status = sample_status();
        let path = Path::new("QUALITY.md");
        let first =
            synchronize_document("# Documento\n\nConteúdo editorial.\n", &status, path).unwrap();
        let mut changed = status.clone();
        changed.cycle.id = "S99".to_owned();
        let second = synchronize_document(&first, &changed, path).unwrap();

        assert!(second.contains("# Documento\n\nConteúdo editorial."));
        assert!(second.contains("**S99**"));
        assert!(!second.contains("**S24** (2026-09-10)"));
        assert_eq!(marker_positions(&second, START_MARKER).len(), 1);
        assert_eq!(marker_positions(&second, END_MARKER).len(), 1);
    }

    #[test]
    fn pointer_block_is_short_and_has_no_wiki_dump() {
        let block = render_pointer_block(&sample_status(), "\n");
        assert!(block.contains("**S24**"));
        assert!(block.contains("STATUS_OPERACIONAL.md"));
        assert!(!block.contains("Motor 1848"));
        assert!(!block.contains("Validação atual"));
        assert!(block.lines().count() <= 4);
    }

    #[test]
    fn generated_markdown_lists_catalog_and_limits() {
        let md = render_status_md(&sample_status());
        assert!(md.contains("<!-- generated by documentation-sync; do not edit -->"));
        assert!(md.contains("sem certificação de produção"));
        assert!(md.contains("NL 0,25"));
        assert!(md.contains("NL 0,75/1,50"));
        assert!(md.contains("SD 0,25/0,50"));
        assert!(md.contains("SD Omaha 0,50"));
        assert!(md.contains("Pineapple 0,50"));
        assert!(md.contains("R$ 150"));
        assert!(!md.contains("Motor 1848"));
    }

    #[test]
    fn preserves_windows_line_endings() {
        let document =
            synchronize_document("# Documento\r\n", &sample_status(), Path::new("QUALITY.md"))
                .unwrap();
        assert!(document.contains("\r\n> **S24**"));
        assert!(!document.replace("\r\n", "").contains('\n'));
    }

    #[test]
    fn rejects_duplicate_or_unbalanced_markers() {
        let malformed = format!("{START_MARKER}\n{START_MARKER}\n{END_MARKER}");
        assert!(
            synchronize_document(&malformed, &sample_status(), Path::new("QUALITY.md")).is_err()
        );
    }

    #[test]
    fn demo_requires_catalog_markers() {
        let demo = format!("# Demo\n\n{START_MARKER}\npointer\n{END_MARKER}\n");
        let error =
            synchronize_document(&demo, &sample_status(), Path::new(DEMO_FILE_NAME)).unwrap_err();
        assert!(error.contains("CASH_CATALOG"));
    }

    #[test]
    fn demo_replaces_catalog_tables() {
        let demo = format!(
            "# Demo\n\n{CASH_START}\nold cash\n{CASH_END}\n\n{MTT_START}\nold mtt\n{MTT_END}\n\n{START_MARKER}\nold\n{END_MARKER}\n"
        );
        let updated =
            synchronize_document(&demo, &sample_status(), Path::new(DEMO_FILE_NAME)).unwrap();
        assert!(updated.contains("| NL 0,25 |"));
        assert!(updated.contains("| Texas Hold’em |"));
        assert!(!updated.contains("old cash"));
        assert!(!updated.contains("old mtt"));
    }

    #[test]
    fn rejects_certified_and_automatic_pix() {
        let mut invalid = sample_status();
        invalid.production.certified = true;
        assert!(validate_status(&invalid).is_err());

        invalid = sample_status();
        invalid.pix.automatic_in_production = true;
        assert!(validate_status(&invalid).is_err());
    }

    #[test]
    fn rejects_zero_blinds_and_unknown_variant() {
        let mut invalid = sample_status();
        invalid.cash_tables[0].small_blind_cents = 0;
        assert!(validate_status(&invalid).is_err());

        invalid = sample_status();
        invalid.cash_tables[0].variant = "plo5".to_owned();
        assert!(validate_status(&invalid).is_err());
    }

    #[test]
    fn rejects_schema_v1_and_unknown_fields() {
        let mut invalid = sample_status();
        invalid.schema_version = 1;
        assert!(validate_status(&invalid).is_err());

        let mut extra: serde_json::Value = serde_json::from_str(&sample_json()).unwrap();
        extra
            .as_object_mut()
            .unwrap()
            .insert("validation".to_owned(), serde_json::json!("wiki"));
        assert!(parse_status(&extra.to_string()).is_err());
    }

    #[test]
    fn parses_and_validates_sample_json() {
        let status = parse_status(&sample_json()).unwrap();
        validate_status(&status).unwrap();
    }

    #[test]
    fn real_status_json_validates() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(STATUS_FILE);
        let content = fs::read_to_string(&path).expect("STATUS_OPERACIONAL.json deve existir");
        let status = parse_status(&content).expect("JSON schema v2");
        validate_status(&status).expect("fatos operacionais válidos");
        assert_eq!(status.cash_tables.len(), 5);
        assert!(status
            .cash_tables
            .iter()
            .any(|table| { table.small_blind_cents == 75 && table.big_blind_cents == 150 }));
    }
}
