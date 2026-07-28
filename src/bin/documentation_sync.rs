//! Synchronizes the operational-status block that appears in project documentation.
//!
//! The command deliberately owns only the marked block. Historical, legal, and
//! educational prose remains authored by people and is never rewritten here.

use serde::Deserialize;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

const STATUS_FILE: &str = "Documentacao/STATUS_OPERACIONAL.json";
const START_MARKER: &str = "<!-- DOCUMENTATION_SYNC:START -->";
const END_MARKER: &str = "<!-- DOCUMENTATION_SYNC:END -->";

#[derive(Debug, Deserialize)]
struct OperationalStatus {
    schema_version: u32,
    reviewed_on: String,
    cycle: String,
    production: String,
    validation: String,
    pix: String,
    table_ownership: String,
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
    let documents = markdown_documents(&root.join("Documentacao"))?;

    let mut stale_documents = Vec::new();
    for path in &documents {
        let original = fs::read_to_string(&path)
            .map_err(|error| format!("não foi possível ler {}: {error}", path.display()))?;
        let updated = synchronize_document(&original, &status)?;
        let relative_path = path
            .strip_prefix(&root)
            .map_err(|error| format!("caminho documental fora do repositório: {error}"))?
            .display()
            .to_string();

        if updated != original {
            match mode {
                Mode::Check => stale_documents.push(relative_path),
                Mode::Write => fs::write(&path, updated).map_err(|error| {
                    format!("não foi possível atualizar {}: {error}", path.display())
                })?,
            }
        }
    }

    if stale_documents.is_empty() {
        println!(
            "Documentação sincronizada: {} documentos Markdown verificados.",
            documents.len()
        );
        Ok(())
    } else {
        Err(format!(
            "bloco de estado operacional ausente ou divergente em: {}. Rode `cargo run --bin documentation-sync -- --write` e versiona as alterações.",
            stale_documents.join(", ")
        ))
    }
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
            collect_markdown_documents(&path, documents)?;
        } else if file_type.is_file()
            && path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
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
    serde_json::from_str(&content)
        .map_err(|error| format!("{} não é JSON válido: {error}", path.display()))
}

fn validate_status(status: &OperationalStatus) -> Result<(), String> {
    if status.schema_version != 1 {
        return Err(format!(
            "schema_version {} não é suportado (esperado: 1)",
            status.schema_version
        ));
    }

    for (name, value) in [
        ("reviewed_on", &status.reviewed_on),
        ("cycle", &status.cycle),
        ("production", &status.production),
        ("validation", &status.validation),
        ("pix", &status.pix),
        ("table_ownership", &status.table_ownership),
    ] {
        if value.trim().is_empty() {
            return Err(format!("campo obrigatório vazio: {name}"));
        }
    }

    if status.reviewed_on.len() != 10
        || !status
            .reviewed_on
            .chars()
            .enumerate()
            .all(|(index, character)| {
                matches!(index, 4 | 7) && character == '-'
                    || !matches!(index, 4 | 7) && character.is_ascii_digit()
            })
    {
        return Err("reviewed_on deve usar o formato YYYY-MM-DD".to_owned());
    }

    Ok(())
}

fn synchronize_document(original: &str, status: &OperationalStatus) -> Result<String, String> {
    let newline = if original.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let block = render_block(status, newline);
    let start_positions = marker_positions(original, START_MARKER);
    let end_positions = marker_positions(original, END_MARKER);

    match (start_positions.as_slice(), end_positions.as_slice()) {
        ([], []) => {
            let separator = if original.is_empty() || original.ends_with('\n') {
                ""
            } else {
                newline
            };
            Ok(format!("{original}{separator}{newline}{block}{newline}"))
        }
        ([start], [end]) if start < end => {
            let end_of_marker = end + END_MARKER.len();
            Ok(format!(
                "{}{}{}",
                &original[..*start],
                block,
                &original[end_of_marker..]
            ))
        }
        _ => Err("marcadores DOCUMENTATION_SYNC ausentes, duplicados ou fora de ordem".to_owned()),
    }
}

fn marker_positions(content: &str, marker: &str) -> Vec<usize> {
    content
        .match_indices(marker)
        .map(|(index, _)| index)
        .collect()
}

fn render_block(status: &OperationalStatus, newline: &str) -> String {
    [
        START_MARKER.to_owned(),
        format!(
            "> **Estado operacional sincronizado ({}):** {} **{}** {} {} {}",
            status.reviewed_on,
            status.cycle,
            status.production,
            status.validation,
            status.pix,
            status.table_ownership
        ),
        ">".to_owned(),
        "> Fonte canônica: [`STATUS_OPERACIONAL.json`](STATUS_OPERACIONAL.json). Verificação: `cargo run --bin documentation-sync -- --check`.".to_owned(),
        END_MARKER.to_owned(),
    ]
    .join(newline)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_status() -> OperationalStatus {
        OperationalStatus {
            schema_version: 1,
            reviewed_on: "2026-07-28".to_owned(),
            cycle: "Ciclo atual.".to_owned(),
            production: "Sem produção.".to_owned(),
            validation: "Validação atual.".to_owned(),
            pix: "PIX adiado.".to_owned(),
            table_ownership: "Uma réplica.".to_owned(),
        }
    }

    #[test]
    fn inserts_then_replaces_only_the_generated_block() {
        let status = sample_status();
        let first = synchronize_document("# Documento\n\nConteúdo editorial.\n", &status).unwrap();
        let changed_status = OperationalStatus {
            cycle: "Novo ciclo.".to_owned(),
            ..status
        };
        let second = synchronize_document(&first, &changed_status).unwrap();

        assert!(second.contains("# Documento\n\nConteúdo editorial."));
        assert!(second.contains("Novo ciclo."));
        assert!(!second.contains("Ciclo atual."));
        assert_eq!(marker_positions(&second, START_MARKER).len(), 1);
        assert_eq!(marker_positions(&second, END_MARKER).len(), 1);
    }

    #[test]
    fn preserves_windows_line_endings() {
        let document = synchronize_document("# Documento\r\n", &sample_status()).unwrap();
        assert!(document.contains("\r\n> **Estado operacional"));
        assert!(!document.replace("\r\n", "").contains('\n'));
    }

    #[test]
    fn rejects_duplicate_or_unbalanced_markers() {
        let malformed = format!("{START_MARKER}\n{START_MARKER}\n{END_MARKER}");
        assert!(synchronize_document(&malformed, &sample_status()).is_err());
    }

    #[test]
    fn validates_required_fields_and_date_format() {
        let mut invalid = sample_status();
        invalid.reviewed_on = "28-07-2026".to_owned();
        assert!(validate_status(&invalid).is_err());

        invalid = sample_status();
        invalid.pix.clear();
        assert!(validate_status(&invalid).is_err());
    }
}
