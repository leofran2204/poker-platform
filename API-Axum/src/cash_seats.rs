//! Cash-game seat escrow: close an ACTIVE seat and credit the originating wallet.

pub async fn persist_cash_out_seat(
    db: &sqlx::PgPool,
    table_id: uuid::Uuid,
    user_id: &str,
    actor_chips: Option<u64>,
) -> Result<Option<i64>, String> {
    let mut tx = db.begin().await.map_err(|error| error.to_string())?;
    let seat: Option<(uuid::Uuid, i64, String)> = sqlx::query_as(
        "SELECT id, chips, wallet_kind FROM cash_game_seats \
         WHERE table_id = $1 AND user_id = $2::uuid AND status = 'ACTIVE' \
         FOR UPDATE",
    )
    .bind(table_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|error| error.to_string())?;
    let Some((seat_id, stored_chips, wallet_kind)) = seat else {
        tx.commit().await.map_err(|error| error.to_string())?;
        return Ok(None);
    };
    let credit_kind = crate::wallet::WalletKind::from_seat(&wallet_kind);
    let chips = match actor_chips {
        Some(actor_chips) => i64::try_from(actor_chips)
            .map_err(|_| "Actor chip stack exceeds database range".to_string())?,
        None => stored_chips,
    };
    if chips < 0 {
        return Err("Invalid stored chips".to_string());
    }

    sqlx::query(
        "UPDATE cash_game_seats \
         SET chips = $1, status = 'CASHED_OUT', cashed_out_at = NOW() WHERE id = $2",
    )
    .bind(chips)
    .bind(seat_id)
    .execute(&mut *tx)
    .await
    .map_err(|error| error.to_string())?;

    if chips > 0 {
        crate::wallet::credit_wallet(&mut *tx, user_id, chips, credit_kind)
            .await
            .map_err(|error| format!("{error:?}"))?;
        sqlx::query(
            "INSERT INTO cash_game_ledger (user_id, table_id, seat_id, entry_type, amount) \
             VALUES ($1::uuid, $2, $3, 'CASH_OUT', $4)",
        )
        .bind(user_id)
        .bind(table_id)
        .bind(seat_id)
        .bind(chips)
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;
    }

    sqlx::query("INSERT INTO audit_logs (user_id, action, metadata) VALUES ($1, $2, $3)")
        .bind(user_id)
        .bind("SEAT_CASHED_OUT")
        .bind(serde_json::json!({
            "table_id": table_id,
            "seat_id": seat_id,
            "chips_cents": chips,
            "wallet_kind": wallet_kind,
        }))
        .execute(&mut *tx)
        .await
        .map_err(|error| error.to_string())?;

    tx.commit().await.map_err(|error| error.to_string())?;
    Ok(Some(chips))
}
