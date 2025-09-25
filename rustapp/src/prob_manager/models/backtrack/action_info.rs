use crate::history_public::Card;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionInfo {
    Start,
    StartInferred,
    Discard {
        discard: Card,
    }, // Player | Card
    RevealRedraw {
        reveal: Card,
        redraw: Option<Card>,
        relinquish: Option<Card>,
    }, // Player | Reveal Card | Redraw Option<Card>
    ExchangeDraw {
        draw: Vec<Card>,
    }, // Player | Draw Vec<Card> | Return Vec<Card>
    ExchangeChoice {
        relinquish: Vec<Card>,
    }, // Player | Draw Vec<Card> | Return Vec<Card>
    ExchangeDrawChoice {
        draw: Vec<Card>,
        relinquish: Vec<Card>,
    }, // Player | Draw Vec<Card> | Return Vec<Card>
}

impl ActionInfo {
    pub fn public(&self) -> Self {
        match self {
            Self::Start => self.clone(),
            Self::StartInferred => self.clone(),
            Self::Discard { .. } => self.clone(),
            Self::RevealRedraw {
                reveal: revealed, ..
            } => Self::RevealRedraw {
                reveal: *revealed,
                redraw: None,
                relinquish: None,
            },
            Self::ExchangeDrawChoice { .. } => Self::ExchangeDrawChoice {
                draw: Vec::with_capacity(2),
                relinquish: Vec::with_capacity(2),
            },
            Self::ExchangeDraw { .. } => Self::ExchangeDraw {
                draw: Vec::with_capacity(2),
            },
            Self::ExchangeChoice { .. } => Self::ExchangeChoice {
                relinquish: Vec::with_capacity(2),
            },
        }
    }
    /// All private information is known
    pub fn fully_known(&self) -> bool {
        match self {
            Self::Start => true,
            Self::StartInferred => true,
            Self::Discard { .. } => true,
            Self::RevealRedraw { redraw, .. } => redraw.is_some(),
            Self::ExchangeDrawChoice { draw, relinquish } => {
                draw.len() == 2 && relinquish.len() == 2
            }
            Self::ExchangeDraw { draw } => draw.len() == 2,
            Self::ExchangeChoice { .. } => {
                unimplemented!("This is indeterminate as we need to know if player has 1 life or 2")
            }
        }
    }
    /// At least some private information known, or no private information to know
    pub fn partially_known(&self) -> bool {
        match self {
            Self::Start => true,
            Self::StartInferred => true,
            Self::Discard { .. } => true,
            Self::RevealRedraw { redraw, .. } => redraw.is_some(),
            Self::ExchangeDrawChoice { draw, relinquish } => {
                !draw.is_empty() || !relinquish.is_empty()
            }
            Self::ExchangeDraw { draw } => !draw.is_empty(),
            Self::ExchangeChoice { relinquish } => !relinquish.is_empty(),
        }
    }
    /// No private information is known
    pub fn fully_unknown(&self) -> bool {
        match self {
            Self::Start => false,
            Self::StartInferred => false,
            Self::Discard { .. } => false,
            Self::RevealRedraw { redraw, .. } => redraw.is_none(),
            Self::ExchangeDrawChoice { draw, relinquish } => {
                draw.is_empty() && relinquish.is_empty()
            }
            Self::ExchangeDraw { draw } => draw.is_empty(),
            Self::ExchangeChoice { relinquish } => relinquish.is_empty(),
        }
    }
    pub fn name(&self) -> ActionInfoName {
        match self {
            Self::Start => ActionInfoName::Start,
            Self::StartInferred => ActionInfoName::StartInferred,
            Self::Discard { .. } => ActionInfoName::Discard,
            Self::RevealRedraw { .. } => ActionInfoName::RevealRedraw,
            Self::ExchangeDrawChoice { .. } => ActionInfoName::ExchangeDrawChoice,
            Self::ExchangeDraw { .. } => ActionInfoName::ExchangeDraw,
            Self::ExchangeChoice { .. } => ActionInfoName::ExchangeChoice,
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionInfoName {
    Start,
    StartInferred,
    Discard,      // Player | Card
    RevealRedraw, // Player | Reveal Card | Redraw Option<Card>
    ExchangeDraw,
    ExchangeChoice,
    ExchangeDrawChoice, // Player | Draw Vec<Card> | Return Vec<Card>
}
