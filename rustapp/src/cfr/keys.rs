use std::hash::Hasher;
use std::hash::Hash;
use std::ops::Index;
use crate::history_public::Card;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
// TODO: Make BRKey key = player_id * 15 + infostate! still a u8
pub struct BRKey { 
    player_id: u8, //TODO: Change to u8
    infostate: Infostate,
}
pub const MAX_NUM_BRKEY: usize = 6 * 15; 
// pub const INFOSTATES: [&str; 15] = ["AA", "AB", "AC", "AD", "AE", "BB", "BC", "BD", "BE", "CC", "CD", "CE", "DD", "DE", "EE"];
pub const INFOSTATES: [&Infostate; 15] = [&Infostate::AA, &Infostate::AB, &Infostate::AC, &Infostate::AD, &Infostate::AE, &Infostate::BB, &Infostate::BC, &Infostate::BD, &Infostate::BE, &Infostate::CC, &Infostate::CD, &Infostate::CE, &Infostate::DD, &Infostate::DE, &Infostate::EE];
impl BRKey {
    pub fn new(player_id: u8, infostate: &Infostate) -> Self {
        debug_assert!(player_id < 6, "Invalid player_id of {player_id}");
        BRKey {
            player_id,
            infostate: *infostate,
        }
    }
    pub fn set_infostate(&mut self, new_infostate: &Infostate) {
        self.infostate = *new_infostate;
    }
    pub fn set_player_id(&mut self, new_player_id: u8) {
        self.player_id = new_player_id;
    }
    pub fn player_id(&self) -> u8 {
        self.player_id
    }
    pub fn infostate(&self) -> Infostate {
        self.infostate.clone()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MSKey {
    player_id: u8, //TODO: change to u8
    path: String,
}

impl MSKey {
    pub fn new(player_id: u8, path: &str) -> Self {
        debug_assert!(player_id < 6, "Invalid player_id of {player_id}");
        MSKey {
            player_id,
            path: path.to_string(),
        }
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn player_id(&self) -> u8 {
        self.player_id
    }
    pub fn set_path(&mut self, new_path: &str) {
        self.path = new_path.to_string();
    }
    pub fn set_player_id(&mut self, new_player_id: u8) {
        self.player_id = new_player_id;
    }
}

impl Hash for MSKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.player_id.hash(state);
        self.path.hash(state);
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Hash, Copy ,Clone)]
pub enum Infostate {
    AA = 0,
    AB = 1,
    AC = 2,
    AD = 3,
    AE = 4,
    BB = 5,
    BC = 6,
    BD = 7,
    BE = 8,
    CC = 9,
    CD = 10,
    CE = 11,
    DD = 12,
    DE = 13,
    EE = 14,
}

impl Index<usize> for Infostate {
    type Output = str;
    fn index(&self, index: usize) -> &Self::Output {
        const INFOSTATE_STRINGS: [&str; 15] = [
            "AA", "AB", "AC", "AD", "AE", "BB", "BC", "BD", "BE", "CC", "CD", "CE", "DD", "DE", "EE",
        ];
        let infostate_str = INFOSTATE_STRINGS[*self as usize];
        &infostate_str[index..index + 1]
    }
}


impl Infostate {
    pub fn contains(&self, card_str: &str) -> bool {
        self[0] == *card_str || self[1] == *card_str
    }
    pub fn from_str(str: &str) -> Self {
        match str {
            "AA" => Infostate::AA,
            "AB" => Infostate::AB,
            "AC" => Infostate::AC,
            "AD" => Infostate::AD,
            "AE" => Infostate::AE,
            "BB" => Infostate::BB,
            "BC" => Infostate::BC,
            "BD" => Infostate::BD,
            "BE" => Infostate::BE,
            "CC" => Infostate::CC,
            "CD" => Infostate::CD,
            "CE" => Infostate::CE,
            "DD" => Infostate::DD,
            "DE" => Infostate::DE,
            "EE" => Infostate::EE,
            "BA" => Infostate::AB,
            "CA" => Infostate::AC,
            "DA" => Infostate::AD,
            "EA" => Infostate::AE,
            "CB" => Infostate::BC,
            "DB" => Infostate::BD,
            "EB" => Infostate::BE,
            "DC" => Infostate::CD,
            "EC" => Infostate::CE,
            "ED" => Infostate::DE,
            _ => panic!("Invalid String"),
        }
    }
    pub fn cards_to_enum(card_0: &Card, card_1: &Card) -> Self {
        let card_0_str: &str = card_0.card_to_str(); 
        let card_1_str: &str = card_1.card_to_str(); 
        let mut chars: Vec<char> = vec![card_0_str.chars().next().unwrap(), card_1_str.chars().next().unwrap()];
        chars.sort_unstable();
        match chars[0] {
            'A' => {
                return match chars[1] {
                    'A' => Infostate::AA,
                    'B' => Infostate::AB,
                    'C' => Infostate::AC,
                    'D' => Infostate::AD,
                    'E' => Infostate::AE,
                    _ => panic!("inappropriate card provided"),
                };
            },
            'B' => {
                return match chars[1] {
                    'B' => Infostate::BB,
                    'C' => Infostate::BC,
                    'D' => Infostate::BD,
                    'E' => Infostate::BE,
                    _ => panic!("inappropriate card provided"),
                };
            },
            'C' => {
                return match chars[1] {
                    'C' => Infostate::CC,
                    'D' => Infostate::CD,
                    'E' => Infostate::CE,
                    _ => panic!("inappropriate card provided"),
                };
            },
            'D' => {
                return match chars[1] {
                    'D' => Infostate::DD,
                    'E' => Infostate::DE,
                    _ => panic!("inappropriate card provided"),
                };
            },
            'E' => {
                return match chars[1] {
                    'E' => Infostate::EE,
                    _ => panic!("inappropriate card provided"),
                };
            },
            _ => unimplemented!("card beyond E provided"),
        }

    }
    pub fn to_str(&self) -> &str {
        match self {
            Infostate::AA => "AA",
            Infostate::AB => "AB",
            Infostate::AC => "AC",
            Infostate::AD => "AD",
            Infostate::AE => "AE",
            Infostate::BB => "BB",
            Infostate::BC => "BC",
            Infostate::BD => "BD",
            Infostate::BE => "BE",
            Infostate::CC => "CC",
            Infostate::CD => "CD",
            Infostate::CE => "CE",
            Infostate::DD => "DD",
            Infostate::DE => "DE",
            Infostate::EE => "EE",
            _ => unimplemented!(),
        }
    }
    pub fn to_vec_card(&self) -> Vec<Card> {
        match self {
            Infostate::AA => vec![Card::str_to_card("A"), Card::str_to_card("A")],
            Infostate::AB => vec![Card::str_to_card("A"), Card::str_to_card("B")],
            Infostate::AC => vec![Card::str_to_card("A"), Card::str_to_card("C")],
            Infostate::AD => vec![Card::str_to_card("A"), Card::str_to_card("D")],
            Infostate::AE => vec![Card::str_to_card("A"), Card::str_to_card("E")],
            Infostate::BB => vec![Card::str_to_card("B"), Card::str_to_card("B")],
            Infostate::BC => vec![Card::str_to_card("B"), Card::str_to_card("C")],
            Infostate::BD => vec![Card::str_to_card("B"), Card::str_to_card("D")],
            Infostate::BE => vec![Card::str_to_card("B"), Card::str_to_card("E")],
            Infostate::CC => vec![Card::str_to_card("C"), Card::str_to_card("C")],
            Infostate::CD => vec![Card::str_to_card("C"), Card::str_to_card("D")],
            Infostate::CE => vec![Card::str_to_card("C"), Card::str_to_card("E")],
            Infostate::DD => vec![Card::str_to_card("D"), Card::str_to_card("D")],
            Infostate::DE => vec![Card::str_to_card("D"), Card::str_to_card("E")],
            Infostate::EE => vec![Card::str_to_card("E"), Card::str_to_card("E")],
            _ => unimplemented!(),
        }
    }
}
