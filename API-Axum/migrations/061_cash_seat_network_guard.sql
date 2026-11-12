-- 061: guarda antifraude de assento por rede (Termos de Uso, cláusula 2.3).
-- Antes de criar: maior migration em arquivos e em origin/master = 060; a VPS
-- registrou 60 migrations na publicação de 08/11/2026 (QUALITY.md).
--
-- A API grava, na entrada da mesa cash, impressões HMAC (segredo KYC_DATA_PEPPER,
-- domínio "zero-tilt-seat-network-v1") do IP público do jogador e da sua sub-rede
-- (/24 em IPv4, /64 em IPv6). O endereço em claro nunca é gravado. Endereços não
-- públicos (loopback, rede privada, link-local) ficam sem impressão e não são
-- comparados. As impressões são apagadas quando o assento deixa de estar ativo.
-- Idempotente; não altera migrations anteriores nem dados existentes.

ALTER TABLE cash_game_seats
    ADD COLUMN IF NOT EXISTS ip_fingerprint VARCHAR(64),
    ADD COLUMN IF NOT EXISTS subnet_fingerprint VARCHAR(64);

CREATE INDEX IF NOT EXISTS idx_cash_game_seats_active_subnet
    ON cash_game_seats (table_id, subnet_fingerprint)
    WHERE status = 'ACTIVE' AND subnet_fingerprint IS NOT NULL;

CREATE OR REPLACE FUNCTION clear_cash_seat_network_fingerprints() RETURNS trigger AS $$
BEGIN
    IF NEW.status IS DISTINCT FROM 'ACTIVE' THEN
        NEW.ip_fingerprint := NULL;
        NEW.subnet_fingerprint := NULL;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_clear_cash_seat_network_fingerprints ON cash_game_seats;
CREATE TRIGGER trg_clear_cash_seat_network_fingerprints
    BEFORE UPDATE ON cash_game_seats
    FOR EACH ROW
    EXECUTE FUNCTION clear_cash_seat_network_fingerprints();
