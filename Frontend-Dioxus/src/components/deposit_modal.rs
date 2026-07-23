// deposit_modal.rs — Componente Modal Dioxus para Depósito PIX Instantâneo
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct DepositModalProps {
    pub is_open: bool,
    pub on_close: EventHandler<()>,
}

#[allow(non_snake_case)]
pub fn DepositModal(props: DepositModalProps) -> Element {
    let mut amount = use_signal(|| "50.00".to_string());
    let mut pix_code = use_signal(|| "".to_string());
    let mut qr_code = use_signal(|| "".to_string());
    let mut is_loading = use_signal(|| false);
    let mut is_copied = use_signal(|| false);

    if !props.is_open {
        return rsx! {};
    }

    let generate_pix = move |_| {
        is_loading.set(true);
        // Simulação de geração de PIX via API Axum
        let simulated_code = format!(
            "00020126580014BR.GOV.BCB.PIX0136poker-platform-dep-{}5204000053039865405{}5802BR5914POKER_PLATFORM6009SAO_PAULO6304ABCD",
            amount(), amount()
        );
        let simulated_qr = "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='150' height='150'><rect width='150' height='150' fill='%23111827'/><text x='15' y='80' fill='%2310b981' font-size='14'>PIX QR CODE</text></svg>".to_string();

        pix_code.set(simulated_code);
        qr_code.set(simulated_qr);
        is_loading.set(false);
    };

    let copy_pix = move |_| {
        is_copied.set(true);
    };

    rsx! {
        div {
            class: "fixed inset-0 bg-black/80 flex items-center justify-center z-50 p-4",
            div {
                class: "bg-gray-900 border border-green-500/30 rounded-xl max-w-md w-full p-6 text-white shadow-2xl space-y-4",
                div {
                    class: "flex justify-between items-center border-b border-gray-800 pb-3",
                    h3 { class: "text-xl font-bold text-green-400 flex items-center gap-2", "💵 Depósito Instantâneo via PIX" }
                    button {
                        class: "text-gray-400 hover:text-white font-bold text-xl",
                        onclick: move |_| props.on_close.call(()),
                        "✕"
                    }
                }

                div { class: "space-y-3",
                    label { class: "block text-sm font-medium text-gray-300", "Valor do Depósito (R$):" }
                    div { class: "grid grid-cols-4 gap-2 mb-2",
                        button { class: "bg-gray-800 hover:bg-gray-700 py-1.5 rounded text-sm font-semibold", onclick: move |_| amount.set("20.00".to_string()), "R$ 20" }
                        button { class: "bg-gray-800 hover:bg-gray-700 py-1.5 rounded text-sm font-semibold text-green-400 border border-green-500/40", onclick: move |_| amount.set("50.00".to_string()), "R$ 50" }
                        button { class: "bg-gray-800 hover:bg-gray-700 py-1.5 rounded text-sm font-semibold", onclick: move |_| amount.set("100.00".to_string()), "R$ 100" }
                        button { class: "bg-gray-800 hover:bg-gray-700 py-1.5 rounded text-sm font-semibold", onclick: move |_| amount.set("500.00".to_string()), "R$ 500" }
                    }
                    input {
                        class: "w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-white font-mono focus:outline-none focus:border-green-500",
                        value: "{amount}",
                        oninput: move |e| amount.set(e.value())
                    }
                }

                if !pix_code().is_empty() {
                    div { class: "bg-gray-950 p-4 rounded-lg border border-gray-800 text-center space-y-3",
                        p { class: "text-xs text-gray-400 font-semibold", "Escaneie o QRCode ou Copie o código PIX:" }
                        div { class: "flex justify-center",
                            img { class: "w-36 h-36 rounded border border-green-500/50", src: "{qr_code}" }
                        }
                        div { class: "bg-gray-900 p-2 rounded text-xs font-mono text-gray-300 break-all select-all border border-gray-800", "{pix_code}" }
                        button {
                            class: "w-full bg-green-600 hover:bg-green-500 py-2 rounded-lg font-bold transition flex items-center justify-center gap-2",
                            onclick: copy_pix,
                            if is_copied() { "✅ Código PIX Copiado!" } else { "📋 Copiar Código PIX" }
                        }
                    }
                } else {
                    button {
                        class: "w-full bg-gradient-to-r from-green-600 to-emerald-500 hover:from-green-500 hover:to-emerald-400 py-3 rounded-lg font-bold text-lg shadow-lg transition",
                        disabled: is_loading(),
                        onclick: generate_pix,
                        if is_loading() { "Gerando PIX..." } else { "Gerar QRCode PIX" }
                    }
                }
            }
        }
    }
}
