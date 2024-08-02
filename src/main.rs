#![allow(dead_code)]

// --- Dice ---

// return the expected_value of a die roll with the given number of faces
macro_rules! expected_value {
    ($n:literal) => {
        paste::item! {
            #[allow(non_upper_case_globals)]
            const [<d $n>]:f32 =
                (1.0 + $n as f32) / 2.0 ;
        }
    };
}

expected_value!(4);
expected_value!(6);
expected_value!(8);
expected_value!(10);
expected_value!(12);

// --- Traits ---

trait ExpectedDamage {
    fn expected_damage(&self, ac: i32) -> f32;
}

// --- Types ---

#[derive(Default, Debug, Copy, Clone, PartialEq)]
struct Damage {
    dmg: f32,
    // not multiplied on crit
    fixed: i32,
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
struct Attack {
    // bonus to hit chance
    hit: i32,

    dmg: Damage,
    crit: Damage,
}

#[derive(Default, Debug, Clone, PartialEq)]
struct Turn {
    action: Vec<Attack>,
    bonus_action: Vec<Attack>,

    once_on_hit: Damage,
}

// --- Methods ---

impl Damage {
    fn hit(&self) -> f32 {
        self.dmg + self.fixed as f32
    }

    // critical hit doubles non-fixed damage
    fn crit(&self) -> f32 {
        self.dmg + self.hit()
    }
}

impl Attack {
    // excludes natural 20, treats natural 1 as a miss
    fn hit_chance(&self, ac: i32) -> f32 {
        18.min(0.max(20 + self.hit - ac)) as f32 / 20.0
    }

    fn sharpshooter(&self) -> Attack {
        *self
            + Attack {
                hit: -5,
                dmg: Damage {
                    fixed: 10,
                    ..Default::default()
                },
                ..Default::default()
            }
    }
}

impl Turn {
    // Favored Foe
    fn foe(&self) -> Turn {
        Turn {
            action: self.action.clone(),
            bonus_action: self.bonus_action.clone(),

            once_on_hit: self.once_on_hit
                + Damage {
                    dmg: d4,
                    ..Default::default()
                },
        }
    }
}

// --- Trait Methods ---

use core::ops::Add;

impl Add for Damage {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            dmg: self.dmg + other.dmg,
            fixed: self.fixed + other.fixed,
        }
    }
}

impl Add for Attack {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Attack {
            hit: self.hit + other.hit,
            dmg: self.dmg + other.dmg,
            crit: self.crit + other.crit,
        }
    }
}

impl Add<Damage> for Attack {
    type Output = Self;

    fn add(self, dmg: Damage) -> Self {
        Attack {
            hit: self.hit,
            dmg: self.dmg + dmg,
            crit: self.crit,
        }
    }
}

impl Add<Attack> for Turn {
    type Output = Self;

    fn add(self, atk: Attack) -> Self {
        Turn {
            action: self.action.clone().into_iter().map(|a| a + atk).collect(),
            bonus_action: self
                .bonus_action
                .clone()
                .into_iter()
                .map(|a| a + atk)
                .collect(),
            once_on_hit: self.once_on_hit,
        }
    }
}

impl Add<Damage> for Turn {
    type Output = Self;

    fn add(self, dmg: Damage) -> Self {
        Turn {
            action: self.action.clone().into_iter().map(|a| a + dmg).collect(),
            bonus_action: self
                .bonus_action
                .clone()
                .into_iter()
                .map(|a| a + dmg)
                .collect(),
            once_on_hit: self.once_on_hit,
        }
    }
}

impl ExpectedDamage for Attack {
    fn expected_damage(&self, ac: i32) -> f32 {
        self.hit_chance(ac) * self.dmg.hit() + (1.0 / 20.0) * (self.dmg.crit() + self.crit.crit())
    }
}

impl ExpectedDamage for Turn {
    fn expected_damage(&self, ac: i32) -> f32 {
        let mut total = 0.0;
        let mut miss = 1.0;
        let mut first_crit = 0.0;

        let crit_chance = 1.0 / 20.0;

        for d in self.action.iter().chain(self.bonus_action.iter()) {
            total += d.expected_damage(ac);
            first_crit += crit_chance * miss;
            miss *= 1.0 - (d.hit_chance(ac) + crit_chance);
        }

        total += (1.0 - miss) * self.once_on_hit.hit();
        total += first_crit * self.once_on_hit.dmg;

        total
    }
}

// --- Methods ---

fn main() {
    // attacks
    let crossbow = Attack {
        hit: 12,
        dmg: Damage {
            dmg: 1.0 * d6,
            fixed: 6,
        },
        crit: Damage {
            dmg: 1.0 * d6,
            fixed: 0,
        },
    };

    let sharp = crossbow.sharpshooter();

    let longsword = Attack {
        hit: 10,
        dmg: Damage { dmg: d10, fixed: 6 },
        crit: Damage { dmg: d6, fixed: 0 },
    };

    let unarmed = Attack {
        hit: 9,
        dmg: Damage {
            dmg: 1.0 * d6,
            fixed: 5,
        },
        crit: Damage { dmg: 0.0, fixed: 0 },
    };

    // turns
    let crossbow = Turn {
        action: vec![crossbow; 2],
        bonus_action: vec![crossbow],
        ..Default::default()
    };

    let sharp = Turn {
        action: vec![sharp; 2],
        bonus_action: vec![sharp],
        ..Default::default()
    };

    let melee = Turn {
        action: vec![longsword; 2],
        bonus_action: vec![unarmed; 2],
        ..Default::default()
    };

    // what is compared
    let turns = vec![crossbow, sharp, melee];

    let foe_turns: Vec<Turn> = turns.into_iter().map(|x| x.foe()).collect();

    for i in 14..=28 {
        print!(" {} ", i);

        for t in &foe_turns {
            print!(" | {:>5.2}", t.expected_damage(i));
        }

        println!();
    }
}

// --- Tests ---

#[cfg(test)]
mod tests {
    use crate::d10;
    use crate::d4;
    use crate::d6;
    use crate::Attack;
    use crate::Damage;
    use crate::ExpectedDamage;
    use crate::Turn;

    // Dice

    #[test]
    fn test_dice() {
        assert_eq!(d4, 2.5);
        assert_eq!(d10, 5.5);
    }

    // Damage

    #[test]
    fn test_dmg_hit() {
        let dmg = Damage { dmg: 1.0, fixed: 1 };

        assert_eq!(dmg.hit(), 2.0);
    }

    #[test]
    fn test_dmg_crit() {
        let dmg = Damage { dmg: 1.0, fixed: 1 };

        assert_eq!(dmg.crit(), 3.0);
    }

    #[test]
    fn test_dmg_add() {
        assert_eq!(
            Damage { dmg: 1.0, fixed: 1 } + Damage { dmg: 1.0, fixed: 1 },
            Damage { dmg: 2.0, fixed: 2 }
        );
    }

    // Attack

    #[test]
    fn test_attack_add() {
        let dmg = Damage { dmg: 1.0, fixed: 1 };
        let dbl_dmg = Damage { dmg: 2.0, fixed: 2 };

        let atk = Attack {
            hit: 1,
            dmg: dmg,
            crit: dmg,
        };

        assert_eq!(
            atk + atk,
            Attack {
                hit: 2,
                dmg: dbl_dmg,
                crit: dbl_dmg
            }
        );
    }

    #[test]
    fn test_attack_add_damage() {
        let dmg = Damage { dmg: 1.0, fixed: 1 };
        let dbl_dmg = Damage { dmg: 2.0, fixed: 2 };

        let atk = Attack {
            hit: 1,
            dmg: dmg,
            crit: dmg,
        };

        assert_eq!(
            atk + dmg,
            Attack {
                hit: 1,
                dmg: dbl_dmg,
                crit: dmg
            }
        );
    }

    #[test]
    fn test_attack_sharpshooter() {
        let dmg = Damage { dmg: 1.0, fixed: 1 };
        let atk = Attack {
            hit: 10,
            dmg: dmg,
            crit: dmg,
        };

        let sharp = Attack {
            hit: 5,
            dmg: Damage {
                dmg: 1.0,
                fixed: 11,
            },
            crit: dmg,
        };

        assert_eq!(atk.sharpshooter(), sharp);
    }

    #[test]
    fn test_attack_fixed_dmg() {
        let atk = Attack {
            dmg: Damage {
                dmg: 0.0,
                fixed: 20,
            },
            ..Default::default()
        };

        assert_eq!(atk.expected_damage(11), 10.0);
        assert_eq!(atk.expected_damage(20), 1.0);
        assert_eq!(atk.expected_damage(21), 1.0);

        assert_eq!(atk.expected_damage(2), 19.0);
    }

    #[test]
    //#[ignore]
    fn test_attack_fumble() {
        let atk = Attack {
            dmg: Damage {
                dmg: 0.0,
                fixed: 20,
            },
            ..Default::default()
        };
        assert_eq!(atk.expected_damage(1), 19.0);
    }

    #[test]
    fn test_attack_hit_bonus() {
        let atk = Attack {
            hit: 5,
            dmg: Damage {
                dmg: 0.0,
                fixed: 20,
            },
            ..Default::default()
        };

        assert_eq!(atk.expected_damage(11), 15.0);
        assert_eq!(atk.expected_damage(21), 5.0);
    }

    #[test]
    fn test_attack_dmg_crit() {
        let atk = Attack {
            dmg: Damage {
                dmg: 20.0,
                fixed: 0,
            },
            ..Default::default()
        };

        assert_eq!(atk.expected_damage(11), 11.0);

        let atk = Attack {
            dmg: Damage {
                dmg: 20.0,
                fixed: 20,
            },
            ..Default::default()
        };

        assert_eq!(atk.expected_damage(11), 21.0);
    }

    #[test]
    fn test_attack_crit_only_dmg() {
        let atk = Attack {
            crit: Damage {
                dmg: 10.0,
                fixed: 0,
            },
            ..Default::default()
        };

        assert_eq!(atk.expected_damage(11), 1.0);

        let atk = Attack {
            crit: Damage {
                dmg: 0.0,
                fixed: 20,
            },
            ..Default::default()
        };

        assert_eq!(atk.expected_damage(11), 1.0);
    }

    #[test]
    fn test_dmg_dpr_calc() {
        let crossbow = crate::Attack {
            hit: 12,
            dmg: Damage {
                dmg: 1.0 * d6,
                fixed: 6,
            },
            crit: Damage {
                dmg: 1.0 * d6,
                fixed: 0,
            },
        };

        // https://rpgbot.net/dnd5/tools/dpr-calculator/
        assert_eq!(crossbow.expected_damage(17), 8.125);

        let rando = crate::Attack {
            hit: 8,
            dmg: Damage {
                dmg: 2.0 * d6,
                fixed: 5,
            },
            crit: Damage {
                dmg: 1.0 * d4,
                fixed: 3,
            },
        };

        assert_eq!(rando.expected_damage(16), 8.55)
    }

    // Turn

    #[test]
    fn test_turn_actions() {
        let atk = Attack {
            dmg: Damage {
                dmg: 20.0,
                fixed: 20,
            },
            ..Default::default()
        };

        let turn = Turn {
            action: vec![atk],
            ..Default::default()
        };

        assert_eq!(turn.expected_damage(11), atk.expected_damage(11));

        let turn = Turn {
            action: vec![atk; 2],
            ..Default::default()
        };

        assert_eq!(turn.expected_damage(11), 2.0 * atk.expected_damage(11));
    }

    #[test]
    fn test_turn_bonus_actions() {
        let atk = Attack {
            dmg: Damage {
                dmg: 20.0,
                fixed: 20,
            },
            ..Default::default()
        };

        let turn = Turn {
            bonus_action: vec![atk],
            ..Default::default()
        };

        assert_eq!(turn.expected_damage(11), atk.expected_damage(11));

        let turn = Turn {
            bonus_action: vec![atk; 2],
            ..Default::default()
        };

        assert_eq!(turn.expected_damage(11), 2.0 * atk.expected_damage(11));
    }

    #[test]
    fn test_turn_actions_and_bonus_actions() {
        let atk = Attack {
            dmg: Damage {
                dmg: 20.0,
                fixed: 20,
            },
            ..Default::default()
        };

        let turn = Turn {
            action: vec![atk; 2],
            bonus_action: vec![atk; 2],
            ..Default::default()
        };

        assert_eq!(turn.expected_damage(11), 4.0 * atk.expected_damage(11));
    }

    #[test]
    fn test_turn_add_damage() {
        let dmg = Damage { dmg: 1.0, fixed: 1 };
        let dbl_dmg = Damage { dmg: 2.0, fixed: 2 };

        let atk = Attack {
            hit: 1,
            dmg: dmg,
            crit: dmg,
        };

        let doublish_atk = Attack {
            hit: 1,
            dmg: dbl_dmg,
            crit: dmg,
        };

        let turn = Turn {
            action: vec![atk; 2],
            bonus_action: vec![atk; 3],
            once_on_hit: dmg,
        };

        assert_eq!(
            turn + dmg,
            Turn {
                action: vec![doublish_atk; 2],
                bonus_action: vec![doublish_atk; 3],
                once_on_hit: dmg
            }
        )
    }

    #[test]
    fn test_turn_add_attack() {
        let dmg = Damage { dmg: 1.0, fixed: 1 };
        let dbl_dmg = Damage { dmg: 2.0, fixed: 2 };

        let atk = Attack {
            hit: 1,
            dmg: dmg,
            crit: dmg,
        };

        let dbl_atk = Attack {
            hit: 2,
            dmg: dbl_dmg,
            crit: dbl_dmg,
        };

        let turn = Turn {
            action: vec![atk; 2],
            bonus_action: vec![atk; 3],
            once_on_hit: dmg,
        };

        assert_eq!(
            turn + atk,
            Turn {
                action: vec![dbl_atk; 2],
                bonus_action: vec![dbl_atk; 3],
                once_on_hit: dmg
            }
        )
    }

    #[test]
    fn test_turn_once_on_hit_fixed_one_attack() {
        let turn = Turn {
            action: vec![Attack {
                hit: 20,
                ..Default::default()
            }],
            once_on_hit: Damage {
                fixed: 20,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(turn.expected_damage(0), 19.0);
    }

    #[test]
    fn test_turn_once_on_hit_fixed_multiple_attacks() {
        let turn = Turn {
            action: vec![
                Attack {
                    hit: 0,
                    ..Default::default()
                };
                2
            ],
            once_on_hit: Damage {
                fixed: 20,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            format!("{:.4}", turn.expected_damage(20)),
            format!("{:.4}", 1.95)
        );

        let turn = Turn {
            action: vec![
                Attack {
                    hit: 0,
                    ..Default::default()
                };
                4
            ],
            once_on_hit: Damage {
                fixed: 20,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            format!("{:.2}", turn.expected_damage(20)),
            format!("{:.2}", 3.71)
        );
    }

    #[test]
    fn test_turn_once_on_hit_crit_one_attack() {
        let turn = Turn {
            action: vec![Attack {
                hit: 20,
                ..Default::default()
            }],
            once_on_hit: Damage {
                dmg: 20.0,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(turn.expected_damage(0), 20.0);
    }

    #[test]
    fn test_turn_once_on_hit_crit_multiple_attacks() {
        let turn = Turn {
            action: vec![
                Attack {
                    hit: 0,
                    ..Default::default()
                };
                2
            ],
            once_on_hit: Damage {
                dmg: 20.0,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            format!("{:.4}", turn.expected_damage(20)),
            format!("{:.4}", 2.0 * 1.95)
        );

        let turn = Turn {
            action: vec![
                Attack {
                    hit: 0,
                    ..Default::default()
                };
                4
            ],
            once_on_hit: Damage {
                dmg: 20.0,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(
            format!("{:.2}", turn.expected_damage(20)),
            format!("{:.2}", 2.0 * 3.71)
        );
    }

    #[test]
    fn test_turn_foe() {
        let turn = Turn {
            action: vec![Attack {
                ..Default::default()
            }],
            ..Default::default()
        }
        .foe();

        assert_eq!(
            format!("{:.4}", turn.expected_damage(20)),
            format!("{:.4}", (1.0 / 20.0) * 2.0 * d4)
        );
    }
}
