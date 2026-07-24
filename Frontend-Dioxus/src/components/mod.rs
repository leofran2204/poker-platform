//! Componentes reutilizáveis da mesa de poker.
//!
//! Cada componente é uma peça visual isolada que pode ser combinada
//! para construir a interface completa da mesa de poker.
//!
//! Componentes de mesa:
//! - [`card`] — Carta individual (face ou verso)
//! - [`community_cards`] — Cartas comunitárias no centro da mesa
//! - [`avatar`] — Avatar/informações de um jogador
//! - [`seat`] — Assento de um jogador na mesa
//! - [`pot`] — Pote central com valor acumulado
//! - [`action_buttons`] — Botões de ação (Fold/Check/Call/Raise/All-in)
//! - [`table`] — Mesa oval completa integrando todos os componentes
//! - [`notification`] — Componente de Notificações Toast de Confiança ao Jogador
//!
//! Componentes de lobby:
//! - [`table_card`] — Card de mesa individual no lobby
//! - [`lobby_filters`] — Filtros de tipo de jogo e blinds
//! - [`join_button`] — Botão de entrar em uma mesa
//! - [`player_count`] — Indicador visual de jogadores (X/Y)
//! - [`lobby_list`] — Lista de mesas com filtros aplicados

pub mod action_buttons;
pub mod avatar;
pub mod card;
pub mod community_cards;
pub mod deposit_modal;
pub mod join_button;
pub mod lobby_filters;
pub mod lobby_list;
pub mod login_form;
pub mod mfa_input;
pub mod notification;
pub mod player_count;
pub mod pot;
pub mod register_form;
pub mod seat;
pub mod table;
pub mod table_card;
pub mod withdraw_modal;
