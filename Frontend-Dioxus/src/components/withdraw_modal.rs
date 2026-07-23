// withdraw_modal.rs — Componente Modal Dioxus para Saque PIX Instantâneo
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct WithdrawModalProps {
    pub is_open: bool,
    pub on_close: EventHandler<()>,
}

#[allow(non_snake_case)]
pub fn WithdrawModal(props: WithdrawModalProps) -> Element {
    let mut amount = use_signal(|| "100.00".to_string());
    let mut pix_key_type = use_signal(|| "cpf".to_string());
    let mut pix_key = use_signal(|| "".to_string());
    let mut is_loading = use_signal(|| false);
    let mut status_msg = use_signal(|| "".to_string());

    if !props.is_open {
        return rsx! {};
    }

    let execute_withdraw = move |_| {
        if pix_key().trim().is_empty() {
            status_msg.set("⚠️ Chave PIX é obrigatória".to_string());
            return;
        }
        is_loading.set(true);
        status_msg.set("🚀 Processando transferência PIX...".to_string());
    };

    rsx! {
        div {
            class: "fixed inset-0 bg-black/80 flex items-center justify-center z-50 p-4",
            div {
                class: "bg-gray-900 border border-blue-500/30 rounded-xl max-w-md w-full p-6 text-white shadow-2xl space-y-4",
                div {
                    class: "flex justify-between items-center border-b border-gray-800 pb-3",
                    h3 { class: "text-xl font-bold text-blue-400 flex items-center gap-2", "💸 Resgate Instantâneo via PIX" }
                    button {
                        class: "text-gray-400 hover:text-white font-bold text-xl",
                        onclick: move |_| props.on_close.call(()),
                        "✕"
                    }
                }

                div { class: "space-y-3",
                    label { class: "block text-sm font-medium text-gray-300", "Tipo de Chave PIX:" }
                    select {
                        class: "w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white focus:outline-none focus:border-blue-500",
                        value: "{pix_key_type}",
                        onchange: move |e| pix_key_type.set(e.value()),
                        option { value: "cpf", "CPF" }
                        option { value: "email", "E-mail" }
                        option { value: "phone", "Telefone" }
                        option { value: "evp", "Chave Aleatória (EVP)" }
                    }

                    label { class: "block text-sm font-medium text-gray-300", "Chave PIX do Destinatário:" }
                    input {
                        class: "w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white font-mono focus:outline-none focus:border-blue-500",
                        placeholder: "Insira sua chave PIX aqui",
                        value: "{pix_key}",
                        oninput: move |e| pix_key.set(e.value())
                    }

                    label { class: "block text-sm font-medium text-gray-300", "Valor do Resgate (R$):" }
                    input {
                        class: "w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white font-mono focus:outline-none focus:border-blue-500",
                        value: "{amount}",
                        oninput: move |e| amount.set(e.value())
                    }
                }

                if !status_msg().is_empty() {
                    div { class: "p-3 rounded-lg text-sm text-center font-semibold bg-blue-950/60 border border-blue-500/40 text-blue-300", "{status_msg}" }
                }

                button {
                    class: "w-full bg-gradient-to-r from-blue-600 to-indigo-500 hover:from-blue-500 hover:to-indigo-400 py-3 rounded-lg font-bold text-lg shadow-lg transition",
                    disabled: is_loading(),
                    onclick: execute_withdraw,
                    if is_loading() { "Enviando..." } else { "Solicitar Saque PIX" }
                }
            }
        }
    }
}
