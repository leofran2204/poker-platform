pub mod ai_coach;
pub mod equity;

pub use ai_coach::{
    AiCoach, FriendlyCoachAdvice, MathDetail, OpponentRangeEstimate, SimpleAction,
};
pub use equity::{EquityCalculator, EquityResult};
