mod basic;
mod four_kangaroo;
mod three_kangaroo;

pub use basic::Basic as PollardKangarooBasic;
pub use four_kangaroo::PollardFourKangarooSolver as PollardKangarooFour;
pub use three_kangaroo::ThreeKangaroo as PollardKangarooThree;
