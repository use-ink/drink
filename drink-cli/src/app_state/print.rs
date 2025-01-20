use drink::{pallet_revive::ContractResult, sandbox_api::revive_api::decode_debug_buffer};
use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};

use crate::app_state::AppState;

impl AppState {
    pub fn print_command(&mut self, command: &str) {
        self.ui_state.output.push("".into());
        self.ui_state.output.push(
            Span::styled(
                format!("Executing `{command}`"),
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::ITALIC),
            )
            .into(),
        );
    }

    pub fn print(&mut self, msg: &str) {
        self.print_sequence(
            msg.split('\n'),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
    }

    pub fn print_error(&mut self, err: &str) {
        self.print_sequence(
            err.split('\n'),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        );
    }

    fn print_sequence<'a, I: Iterator<Item = &'a str>>(&mut self, seq: I, style: Style) {
        for line in seq {
            self.ui_state
                .output
                .push(Span::styled(line.to_string(), style).into());
        }
    }
}

pub fn format_contract_action<R, E>(result: &ContractResult<R, u128, E>) -> String {
    let mut output = format!(
        "Gas consumed: {:?}\nGas required: {:?}\nDebug buffer:\n",
        result.gas_consumed, result.gas_required
    );

    for line in &decode_debug_buffer(&result.debug_message) {
        output.push_str(&format!("  {line}\n"));
    }

    output
}
