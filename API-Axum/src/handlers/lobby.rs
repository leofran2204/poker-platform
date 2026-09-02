// Lobby handlers — database-authoritative table discovery and cash-game seating.

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;
use crate::game_actor::PlayerCommand;
use crate::middleware::auth::RequireAuth;
use crate::state::AppState;

// ─── Response DTOs ───

#[derive(Debug, Serialize)]
pub struct TableResponse {
    pub id: String,
    pub name: String,
    pub players: u8,
    pub max_players: u8,
    pub small_blind: u64,
    pub big_blind: u64,
    pub min_buy_in: u64,
    pub max_buy_in: u64,
    pub game_type: String,
    /// `play` | `real` — fichas PM não servem em mesas real e vice-versa.
    pub money_mode: String,
    /// `holdem` | `short_deck` | `short_deck_omaha` | `ultimate_pineapple`
    pub poker_variant: String,
}

#[derive(Debug, Serialize)]
pub struct JoinResponse {
    pub seat: u8,
    pub chips: u64,
}

#[derive(Debug, Deserialize)]
pub struct JoinBody {
    pub table_id: String,
    /// Amount moved from wallet to the table escrow, in cents.
    pub buy_in: u64,
    /// `play` (default) uses Play Money cash; `real` uses Jogo Real.
    #[serde(default)]
    pub wallet_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LeaveBody {
    pub table_id: String,
}

#[derive(Debug, Serialize)]
pub struct CashOutResponse {
    pub chips: u64,
}

type TableRow = (
    uuid::Uuid,
    String,
    String,
    i64,
    i64,
    i64,
    i64,
    i16,
    i16,
    String,
    String,
);

fn table_response(
    (
        id,
        name,
        game_type,
        small_blind,
        big_blind,
        min_buy_in,
        max_buy_in,
        max_players,
        current_players,
        money_mode,
        poker_variant,
    ): TableRow,
) -> Result<TableResponse, ApiError> {
    Ok(TableResponse {
        id: id.to_string(),
        name,
        players: u8::try_from(current_players)
            .map_err(|_| ApiError::Internal("Invalid table player count".to_string()))?,
        max_players: u8::try_from(max_players)
            .map_err(|_| ApiError::Internal("Invalid table capacity".to_string()))?,
        small_blind: u64::try_from(small_blind)
            .map_err(|_| ApiError::Internal("Invalid small blind".to_string()))?,
        big_blind: u64::try_from(big_blind)
            .map_err(|_| ApiError::Internal("Invalid big blind".to_string()))?,
        min_buy_in: u64::try_from(min_buy_in)
            .map_err(|_| ApiError::Internal("Invalid minimum buy-in".to_string()))?,
        max_buy_in: u64::try_from(max_buy_in)
            .map_err(|_| ApiError::Internal("Invalid maximum buy-in".to_string()))?,
        game_type: game_type.to_ascii_lowercase(),
        money_mode: if money_mode.eq_ignore_ascii_case("real") {
            "real".into()
        } else {
            "play".into()
        },
        poker_variant: poker_engine::types::PokerVariant::parse(&poker_variant)
            .as_str()
            .to_string(),
    })
}

#[derive(Debug, Deserialize)]
pub struct ListTablesQuery {
    /// `play` | `real` — filtra mesas do modo (padrão: play)
    pub mode: Option<String>,
}

// ─── Handlers ───

/// GET /api/lobby/tables?mode=play|real
pub async fn list_tables(
    State(state): State<AppState>,
    Query(query): Query<ListTablesQuery>,
) -> Result<Json<Vec<TableResponse>>, ApiError> {
    let mode = crate::wallet::WalletMode::parse(query.mode.as_deref());
    let mode_str = mode.as_str();
    let tables: Vec<TableRow> = sqlx::query_as(
        "SELECT id, name, game_type, small_blind, big_blind, min_buy_in, max_buy_in, max_players, current_players, \
                COALESCE(money_mode, 'play') AS money_mode, \
                COALESCE(poker_variant, 'holdem') AS poker_variant \
         FROM tables \
         WHERE visibility = 'public' AND status = 'OPEN' AND current_players < max_players \
           AND COALESCE(money_mode, 'play') = $1 \
         ORDER BY poker_variant, big_blind, name, id",
    )
    .bind(mode_str)
    .fetch_all(&state.db)
    .await?;
    let response = tables
        .into_iter()
        .map(table_response)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(response))
}

/// POST /api/lobby/join
/// Request: `{table_id, buy_in}` → Response: `{seat, chips}`
pub async fn join_table(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Json(body): Json<JoinBody>,
) -> Result<Json<JoinResponse>, ApiError> {
    let table_id = uuid::Uuid::parse_str(&body.table_id)
        .map_err(|_| ApiError::BadRequest("Invalid table id".to_string()))?;
    let buy_in = i64::try_from(body.buy_in)
        .map_err(|_| ApiError::BadRequest("Buy-in is too large".to_string()))?;
    if buy_in <= 0 {
        return Err(ApiError::BadRequest(
            "Buy-in must be greater than zero".to_string(),
        ));
    }

    // Locking the table row serializes seat allocation and capacity checks. The
    // wallet debit and the escrow record are committed atomically below.
    let mut tx = state.db.begin().await?;
    let table: Option<(i64, i64, i16, i16, String, String, String)> = sqlx::query_as(
        "SELECT min_buy_in, max_buy_in, max_players, current_players, visibility, status, \
                COALESCE(money_mode, 'play') \
         FROM tables WHERE id = $1 FOR UPDATE",
    )
    .bind(table_id)
    .fetch_optional(&mut *tx)
    .await?;
    let (
        min_buy_in,
        max_buy_in,
        max_players,
        current_players,
        visibility,
        status,
        table_money_mode,
    ) = table.ok_or_else(|| ApiError::NotFound("Table not found".to_string()))?;

    if visibility != "public" || status != "OPEN" {
        return Err(ApiError::Forbidden(
            "This table is not accepting new players".to_string(),
        ));
    }

    let mode = crate::wallet::WalletMode::parse(body.wallet_mode.as_deref());
    let table_is_real = table_money_mode.eq_ignore_ascii_case("real");
    let mode_is_real = matches!(mode, crate::wallet::WalletMode::Real);
    if table_is_real != mode_is_real {
        return Err(ApiError::BadRequest(
            if table_is_real {
                "Esta mesa é de Jogo Real. Fichas Play Money não podem ser usadas aqui. Mude o modo no header para Jogo Real."
            } else {
                "Esta mesa é de Play Money. Saldo de Jogo Real não entra aqui. Mude o modo no header para Play Money."
            }
            .into(),
        ));
    }
    if min_buy_in == max_buy_in {
        if buy_in != min_buy_in {
            return Err(ApiError::BadRequest(format!(
                "Esta mesa tem frente fixa de {min_buy_in} centavos"
            )));
        }
    } else if buy_in < min_buy_in || buy_in > max_buy_in {
        return Err(ApiError::BadRequest(format!(
            "Buy-in must be between {min_buy_in} and {max_buy_in} cents"
        )));
    }

    // Repeating a join request is idempotent and cannot debit the wallet twice.
    let existing: Option<(i16, i64)> = sqlx::query_as(
        "SELECT seat, chips FROM cash_game_seats \
         WHERE table_id = $1 AND user_id = $2::uuid AND status = 'ACTIVE' \
         FOR UPDATE",
    )
    .bind(table_id)
    .bind(&auth_user.user_id)
    .fetch_optional(&mut *tx)
    .await?;
    if let Some((seat, chips)) = existing {
        return Ok(Json(JoinResponse {
            seat: u8::try_from(seat)
                .map_err(|_| ApiError::Internal("Invalid stored seat".to_string()))?,
            chips: u64::try_from(chips)
                .map_err(|_| ApiError::Internal("Invalid stored chips".to_string()))?,
        }));
    }

    if current_players >= max_players {
        return Err(ApiError::BadRequest("Table is full".to_string()));
    }

    let seat: Option<(i16,)> = sqlx::query_as(
        "SELECT candidate::SMALLINT \
         FROM generate_series(0, $2 - 1) AS candidate \
         WHERE NOT EXISTS ( \
             SELECT 1 FROM cash_game_seats \
             WHERE table_id = $1 AND seat = candidate::SMALLINT AND status = 'ACTIVE' \
         ) \
         ORDER BY candidate LIMIT 1",
    )
    .bind(table_id)
    .bind(i32::from(max_players))
    .fetch_optional(&mut *tx)
    .await?;
    let seat = seat
        .map(|(seat,)| seat)
        .ok_or_else(|| ApiError::BadRequest("Table is full".to_string()))?;

    let kind = crate::wallet::cash_kind_for_mode(mode);
    crate::wallet::ensure_pm_daily_reset(&mut *tx, &auth_user.user_id).await?;
    crate::wallet::debit_wallet(&mut *tx, &auth_user.user_id, buy_in, kind).await?;

    let (seat_id,): (uuid::Uuid,) = sqlx::query_as(
        "INSERT INTO cash_game_seats (table_id, user_id, seat, chips, buy_in, wallet_kind) \
         VALUES ($1, $2::uuid, $3, $4, $4, $5) RETURNING id",
    )
    .bind(table_id)
    .bind(&auth_user.user_id)
    .bind(seat)
    .bind(buy_in)
    .bind(kind.seat_label())
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO cash_game_ledger (user_id, table_id, seat_id, entry_type, amount) \
         VALUES ($1::uuid, $2, $3, 'BUY_IN', $4)",
    )
    .bind(&auth_user.user_id)
    .bind(table_id)
    .bind(seat_id)
    .bind(buy_in)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(JoinResponse {
        seat: u8::try_from(seat)
            .map_err(|_| ApiError::Internal("Allocated seat is invalid".to_string()))?,
        chips: body.buy_in,
    }))
}

/// POST /api/lobby/leave
/// Moves a player's table escrow back to their wallet between hands.
pub async fn leave_table(
    State(state): State<AppState>,
    RequireAuth(auth_user): RequireAuth,
    Json(body): Json<LeaveBody>,
) -> Result<Json<CashOutResponse>, ApiError> {
    let table_id = uuid::Uuid::parse_str(&body.table_id)
        .map_err(|_| ApiError::BadRequest("Invalid table id".to_string()))?;
    let table_key = table_id.to_string();

    let actor_chips = {
        let active_tables = state.active_tables.read().await;
        active_tables.get(&table_key).cloned()
    };
    let actor_chips = if let Some(actor) = actor_chips {
        let (respond_to, response) = tokio::sync::oneshot::channel();
        actor
            .tx_cmd
            .send(PlayerCommand::CashOut {
                player_id: auth_user.user_id.clone(),
                respond_to,
            })
            .await
            .map_err(|_| ApiError::Internal("Table actor is unavailable".to_string()))?;
        response
            .await
            .map_err(|_| ApiError::Internal("Table actor did not respond".to_string()))?
            .map_err(ApiError::BadRequest)?
    } else {
        None
    };

    let mut tx = state.db.begin().await?;
    let seat: Option<(uuid::Uuid, i64, String)> = sqlx::query_as(
        "SELECT id, chips, wallet_kind FROM cash_game_seats \
         WHERE table_id = $1 AND user_id = $2::uuid AND status = 'ACTIVE' \
         FOR UPDATE",
    )
    .bind(table_id)
    .bind(&auth_user.user_id)
    .fetch_optional(&mut *tx)
    .await?;
    let (seat_id, stored_chips, wallet_kind) =
        seat.ok_or_else(|| ApiError::NotFound("No active seat found for this player".to_string()))?;
    let credit_kind = crate::wallet::WalletKind::from_seat(&wallet_kind);
    // PostgreSQL BIGINT is signed while the game engine represents chips as u64.
    // Refuse an impossible engine value rather than wrapping it into a negative
    // database balance during cash-out.
    let chips = match actor_chips {
        Some(actor_chips) => i64::try_from(actor_chips).map_err(|_| {
            ApiError::Internal("Actor chip stack exceeds database range".to_string())
        })?,
        None => stored_chips,
    };
    if chips < 0 {
        return Err(ApiError::Internal("Invalid stored chips".to_string()));
    }

    sqlx::query(
        "UPDATE cash_game_seats \
         SET chips = $1, status = 'CASHED_OUT', cashed_out_at = NOW() WHERE id = $2",
    )
    .bind(chips)
    .bind(seat_id)
    .execute(&mut *tx)
    .await?;
    // A player who lost the full buy-in must still be able to close the seat.
    // The immutable ledger intentionally records only positive transfers.
    if chips > 0 {
        crate::wallet::credit_wallet(&mut *tx, &auth_user.user_id, chips, credit_kind).await?;
        sqlx::query(
            "INSERT INTO cash_game_ledger (user_id, table_id, seat_id, entry_type, amount) \
             VALUES ($1::uuid, $2, $3, 'CASH_OUT', $4)",
        )
        .bind(&auth_user.user_id)
        .bind(table_id)
        .bind(seat_id)
        .bind(chips)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    Ok(Json(CashOutResponse {
        chips: u64::try_from(chips)
            .map_err(|_| ApiError::Internal("Invalid stored chips".to_string()))?,
    }))
}

/// GET /api/lobby/tables/{id}
/// Get a specific table by ID
pub async fn get_table(
    State(state): State<AppState>,
    Path(table_id): Path<String>,
) -> Result<Json<TableResponse>, ApiError> {
    let table_id = uuid::Uuid::parse_str(&table_id)
        .map_err(|_| ApiError::NotFound("Table not found".to_string()))?;
    let table: Option<TableRow> = sqlx::query_as(
        "SELECT id, name, game_type, small_blind, big_blind, min_buy_in, max_buy_in, max_players, current_players, \
                COALESCE(money_mode, 'play') AS money_mode, \
                COALESCE(poker_variant, 'holdem') AS poker_variant \
         FROM tables WHERE id = $1 AND visibility = 'public' AND status = 'OPEN'",
    )
    .bind(table_id)
    .fetch_optional(&state.db)
    .await?;
    let table = table.ok_or_else(|| ApiError::NotFound("Table not found".to_string()))?;

    Ok(Json(table_response(table)?))
}
