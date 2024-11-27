#![forbid(unsafe_code)]

////////////////////////////////////////////////////////////////////////////////

use std::any::Any;
use crate::Move::Cooperated;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundOutcome {
    BothCooperated,
    LeftCheated,
    RightCheated,
    BothCheated,
}

pub struct Game {
    left_score: i32,
    right_score: i32,
    left_agent: Box<dyn Agency>,
    right_agent: Box<dyn Agency>,
    previous_left_move: Move,
    previous_right_move: Move,
}

impl Game {
    pub fn new(left: Box<dyn Agency>, right: Box<dyn Agency>) -> Self {
        Self {
            left_score: 0,
            right_score: 0,
            left_agent: left,
            right_agent: right,
            previous_left_move: Cooperated,
            previous_right_move: Cooperated,
        }
    }

    pub fn left_score(&self) -> i32 {
        self.left_score
    }

    pub fn right_score(&self) -> i32 {
        self.right_score
    }

    pub fn play_round(&mut self) -> RoundOutcome {
        let left_move = self.left_agent.make_turn(self.previous_right_move);
        let right_move = self.right_agent.make_turn(self.previous_left_move);
        self.previous_left_move = left_move;
        self.previous_right_move = right_move;
        match (left_move, right_move) {
            (Move::Cheated, Move::Cheated) => RoundOutcome::BothCheated,
            (Move::Cooperated, Move::Cooperated) => {
                self.left_score += 2;
                self.right_score += 2;
                RoundOutcome::BothCooperated},
            (Move::Cheated, Move::Cooperated) => {
                self.left_score += 3;
                self.right_score -=1;
                RoundOutcome::LeftCheated},
            (Move::Cooperated, Move::Cheated) => {
                self.left_score -= 1;
                self.right_score += 3;
                RoundOutcome::RightCheated
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct CheatingAgent {}

impl Agency for CheatingAgent {
    fn make_turn(&mut self, previous_enemy_move: Move) -> Move {
        Move::Cheated
    }
}


#[derive(Default)]
pub struct CooperatingAgent {}

impl Agency for CooperatingAgent {
    fn make_turn(&mut self, previous_enemy_move: Move) -> Move {
        Move::Cooperated
    }
}
#[derive(Default)]
pub struct GrudgerAgent {
    has_betrayed: bool,
}

impl Agency for GrudgerAgent {
    fn make_turn(&mut self, previous_enemy_move: Move) -> Move {
        match (self.has_betrayed, previous_enemy_move) {
            (true, _) | (_, Move::Cheated) => {
                self.has_betrayed = true;
                Move::Cheated
            }
            (_, _) => {
                Move::Cooperated
            }
        }
    }
}
#[derive(Default)]
pub struct CopycatAgent {
    turn: i32,
}

impl Agency for CopycatAgent {
    fn make_turn(&mut self, previous_enemy_move: Move) -> Move {
        self.turn += 1;
        match self.turn {
            1 => Move::Cooperated,
            _ => previous_enemy_move
        }
    }
}

pub struct DetectiveAgent {
    turn: i32,
    have_been_cheated: bool,
    // TODO: your code goes here.
}

impl Default for DetectiveAgent {
    fn default() -> Self {
        Self{turn: 0, have_been_cheated: false}
    }
}

impl Agency for DetectiveAgent {
    fn make_turn(&mut self, previous_enemy_move: Move) -> Move {
        if self.turn <= 4 && previous_enemy_move == Move::Cheated {
            self.have_been_cheated = true;
        }
        self.turn += 1;
        match (self.turn, self.have_been_cheated) {
            (2, _) => {
                Move::Cheated
            }
            (0..=4, _) => {
                Move::Cooperated
            }
            (_, false) => {
                Move::Cheated
            }
            (_, true) => {
                previous_enemy_move
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Move {
    Cheated,
    Cooperated,
}
pub trait Agency {
    fn make_turn(&mut self, current_move: Move) -> Move;
}