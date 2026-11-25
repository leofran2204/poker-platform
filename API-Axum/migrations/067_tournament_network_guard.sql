-- 067: guarda de rede nos torneios (Termos de Uso, cláusula 2.3).
-- Antes de criar: maior migration em arquivos e em origin/master = 066; a VPS
-- registrou 66 migrations na publicação de 24/11/2026 (QUALITY.md).
--
-- Na inscrição, a API grava a impressão HMAC da sub-rede do jogador (/24 em
-- IPv4, /64 em IPv6; mesma função da migration 061, domínio
-- "zero-tilt-seat-network-v1"). O coordenador usa a impressão para sentar
-- contas da mesma rede em mesas diferentes, e a inscrição é recusada quando já
-- há uma conta da mesma rede em cada mesa. O endereço em claro nunca é gravado.
-- As impressões são apagadas quando o torneio termina ou é cancelado.
-- Idempotente; não altera migrations anteriores nem dados existentes.

ALTER TABLE tournament_players
    ADD COLUMN IF NOT EXISTS subnet_fingerprint VARCHAR(64);

CREATE INDEX IF NOT EXISTS idx_tournament_players_subnet
    ON tournament_players (tournament_id, subnet_fingerprint)
    WHERE subnet_fingerprint IS NOT NULL;

CREATE OR REPLACE FUNCTION clear_tournament_network_fingerprints() RETURNS trigger AS $$
BEGIN
    IF NEW.status IN ('finished', 'cancelled')
       AND OLD.status IS DISTINCT FROM NEW.status THEN
        UPDATE tournament_players
           SET subnet_fingerprint = NULL
         WHERE tournament_id = NEW.id
           AND subnet_fingerprint IS NOT NULL;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_clear_tournament_network_fingerprints ON tournaments;
CREATE TRIGGER trg_clear_tournament_network_fingerprints
    AFTER UPDATE OF status ON tournaments
    FOR EACH ROW
    EXECUTE FUNCTION clear_tournament_network_fingerprints();
