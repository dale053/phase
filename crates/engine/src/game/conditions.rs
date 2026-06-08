//! Shared game-state predicate helpers used by multiple condition evaluators.
//!
//! `eval_condition` is the single authority for context-free `Condition` variants.
//! Context-specific wrapper enums delegate their `Shared { condition }` arms here.

use crate::game::filter::{matches_target_filter, FilterContext};
use crate::game::game_object::GameObject;
use crate::game::quantity::{
    filter_uses_recipient, quantity_expr_uses_object_count, quantity_expr_uses_recipient,
    QuantityContext,
};
use crate::game::speed::has_max_speed;
use crate::types::ability::{
    CommanderOwnership, Condition, ControllerRef, QuantityExpr, SourceIsTappedEval,
};
use crate::types::counter::CounterMatch;
use crate::types::game_state::{DayNight, GameState};
use crate::types::identifiers::ObjectId;
use crate::types::player::PlayerId;
use crate::types::zones::Zone;

pub(crate) fn counter_condition_matches(
    obj: &GameObject,
    counters: &CounterMatch,
    minimum: u32,
    maximum: Option<u32>,
) -> bool {
    let count: u32 = match counters {
        CounterMatch::Any => obj.counters.values().sum(),
        CounterMatch::OfType(ct) => obj.counters.get(ct).copied().unwrap_or(0),
    };
    count >= minimum && maximum.is_none_or(|max| count <= max)
}

pub(crate) fn eval_source_is_tapped_on_battlefield(state: &GameState, source_id: ObjectId) -> bool {
    state
        .objects
        .get(&source_id)
        .is_some_and(|obj| obj.zone == Zone::Battlefield && obj.tapped)
}

pub(crate) fn eval_source_is_tapped(state: &GameState, source_id: ObjectId) -> bool {
    state.objects.get(&source_id).is_some_and(|obj| obj.tapped)
}

pub(crate) fn eval_chosen_label_is(state: &GameState, source_id: ObjectId, label: &str) -> bool {
    state
        .objects
        .get(&source_id)
        .and_then(|obj| obj.chosen_label())
        .is_some_and(|chosen| chosen.eq_ignore_ascii_case(label))
}

pub(crate) fn eval_class_level_ge(state: &GameState, source_id: ObjectId, level: u8) -> bool {
    state
        .objects
        .get(&source_id)
        .and_then(|obj| obj.class_level)
        .is_some_and(|current| current >= level)
}

pub(crate) fn eval_source_in_zone(state: &GameState, source_id: ObjectId, zone: Zone) -> bool {
    state
        .objects
        .get(&source_id)
        .is_some_and(|obj| obj.zone == zone)
}

pub(crate) fn eval_source_is_attacking(state: &GameState, source_id: ObjectId) -> bool {
    state
        .combat
        .as_ref()
        .is_some_and(|combat| combat.attackers.iter().any(|a| a.object_id == source_id))
}

pub(crate) fn eval_no_monarch(state: &GameState) -> bool {
    state.monarch.is_none()
}

pub(crate) fn eval_is_monarch(state: &GameState, controller: PlayerId) -> bool {
    state.monarch == Some(controller)
}

pub(crate) fn eval_has_city_blessing(state: &GameState, controller: PlayerId) -> bool {
    state.city_blessing.contains(&controller)
}

pub(crate) fn eval_source_entered_this_turn(state: &GameState, source_id: ObjectId) -> bool {
    state
        .objects
        .get(&source_id)
        .is_some_and(|obj| obj.entered_battlefield_turn == Some(state.turn_number))
}

pub(crate) fn condition_tree_uses_recipient(condition: &Condition) -> bool {
    match condition {
        Condition::SourceMatchesFilter { filter } => filter_uses_recipient(filter),
        Condition::QuantityComparison { lhs, rhs, .. } => {
            quantity_expr_uses_recipient(lhs) || quantity_expr_uses_recipient(rhs)
        }
        Condition::And { conditions } | Condition::Or { conditions } => {
            conditions.iter().any(condition_tree_uses_recipient)
        }
        Condition::Not { condition } => condition_tree_uses_recipient(condition),
        _ => false,
    }
}

pub(crate) fn condition_tree_uses_object_population(condition: &Condition) -> bool {
    match condition {
        Condition::QuantityComparison { lhs, rhs, .. } => {
            quantity_expr_uses_object_count(lhs) || quantity_expr_uses_object_count(rhs)
        }
        Condition::ControlsCommander { .. } => true,
        Condition::And { conditions } | Condition::Or { conditions } => {
            conditions.iter().any(condition_tree_uses_object_population)
        }
        Condition::Not { condition } => condition_tree_uses_object_population(condition),
        _ => false,
    }
}

pub(crate) fn eval_condition(
    state: &GameState,
    condition: &Condition,
    controller: PlayerId,
    source_id: ObjectId,
    recipient_id: Option<ObjectId>,
    source_is_tapped: SourceIsTappedEval,
) -> bool {
    match condition {
        Condition::And { conditions } => conditions.iter().all(|c| {
            eval_condition(
                state,
                c,
                controller,
                source_id,
                recipient_id,
                source_is_tapped,
            )
        }),
        Condition::Or { conditions } => conditions.iter().any(|c| {
            eval_condition(
                state,
                c,
                controller,
                source_id,
                recipient_id,
                source_is_tapped,
            )
        }),
        Condition::Not { condition } => !eval_condition(
            state,
            condition,
            controller,
            source_id,
            recipient_id,
            source_is_tapped,
        ),
        Condition::HasMaxSpeed => has_max_speed(state, controller),
        Condition::IsMonarch => eval_is_monarch(state, controller),
        Condition::NoMonarch => eval_no_monarch(state),
        Condition::HasCityBlessing => eval_has_city_blessing(state, controller),
        Condition::SourceIsTapped => match source_is_tapped {
            SourceIsTappedEval::OnBattlefield => {
                eval_source_is_tapped_on_battlefield(state, source_id)
            }
            SourceIsTappedEval::RegardlessOfZone => eval_source_is_tapped(state, source_id),
        },
        Condition::SourceMatchesFilter { filter } => matches_target_filter(
            state,
            source_id,
            filter,
            &FilterContext::from_source(state, source_id),
        ),
        Condition::SourceEnteredThisTurn => eval_source_entered_this_turn(state, source_id),
        Condition::WasStartingPlayer { controller: scope } => {
            let subject = match scope {
                ControllerRef::You => controller,
                _ => controller,
            };
            state.current_starting_player == subject
        }
        Condition::SpellCastWithVariantThisTurn { variant } => {
            crate::game::restrictions::spell_cast_with_variant_this_turn(state, variant)
        }
        Condition::ClassLevelGE { level } => eval_class_level_ge(state, source_id, *level),
        Condition::ControlsCommander { ownership } => match ownership {
            CommanderOwnership::Own => {
                crate::game::commander::controls_own_commander(state, controller)
            }
            CommanderOwnership::Any => {
                crate::game::commander::controls_any_commander(state, controller)
            }
        },
        Condition::ChosenLabelIs { label } => eval_chosen_label_is(state, source_id, label),
        Condition::SourceInZone { zone } => eval_source_in_zone(state, source_id, *zone),
        Condition::HasCounters {
            counters,
            minimum,
            maximum,
        } => state
            .objects
            .get(&source_id)
            .map(|obj| counter_condition_matches(obj, counters, *minimum, *maximum))
            .unwrap_or(false),
        Condition::QuantityComparison {
            lhs,
            comparator,
            rhs,
        } => {
            let resolve = |expr: &QuantityExpr| -> i32 {
                crate::game::quantity::resolve_quantity_with_ctx(
                    state,
                    expr,
                    controller,
                    QuantityContext {
                        entering: None,
                        source: source_id,
                        recipient: recipient_id,
                        scoped_player: None,
                    },
                )
            };
            comparator.evaluate(resolve(lhs), resolve(rhs))
        }
        Condition::DayNightIs {
            state: DayNight::Day,
        } => state.day_night == Some(DayNight::Day),
        Condition::DayNightIs {
            state: DayNight::Night,
        } => state.day_night == Some(DayNight::Night),
        Condition::DuringYourTurn => state.active_player == controller,
        Condition::SourceIsAttacking => eval_source_is_attacking(state, source_id),
        Condition::CastVariantPaid { variant } => state
            .objects
            .get(&source_id)
            .is_some_and(|obj| obj.cast_variant_paid.is_some_and(|(v, _)| v == *variant)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::zones::create_object;
    use crate::types::CardId;

    #[test]
    fn tapped_zone_guard_distinguishes_battlefield_vs_graveyard() {
        let mut state = GameState::new_two_player(42);
        let id = create_object(
            &mut state,
            CardId(1),
            PlayerId(0),
            "Test".to_string(),
            Zone::Graveyard,
        );
        state.objects.get_mut(&id).unwrap().tapped = true;
        assert!(!eval_source_is_tapped_on_battlefield(&state, id));
        assert!(eval_source_is_tapped(&state, id));
    }

    #[test]
    fn eval_condition_and_or_not_combinators() {
        let mut state = GameState::new_two_player(42);
        let id = create_object(
            &mut state,
            CardId(3),
            PlayerId(0),
            "Test".to_string(),
            Zone::Battlefield,
        );
        let and = Condition::And {
            conditions: vec![
                Condition::IsMonarch,
                Condition::Not {
                    condition: Box::new(Condition::IsMonarch),
                },
            ],
        };
        assert!(!eval_condition(
            &state,
            &and,
            PlayerId(0),
            id,
            None,
            SourceIsTappedEval::OnBattlefield,
        ));
    }
}
