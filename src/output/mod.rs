pub mod terminal;

use crate::scoring::Scorecard;

pub fn print_scorecard(scorecard: &Scorecard) {
    terminal::render(scorecard);
}
