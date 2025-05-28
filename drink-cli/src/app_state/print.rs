use drink::pallet_revive::ContractResult;
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

pub fn format_contract_action<R>(result: &ContractResult<R, u128>) -> String {
    format!(
        "Gas consumed: {:?}\nGas required: {:?}\n",
        result.gas_consumed, result.gas_required
    )
}
