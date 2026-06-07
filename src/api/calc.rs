//! Pure ingredient-quantity math, shared by every report and unit-tested.
//!
//! All reports scale a recipe's base ingredient quantities by how many
//! base-servings the attending people represent. Keeping the formula here (and
//! out of SQL) means it has a single source of truth that we can test directly.

/// Canonical default per-group serving multipliers.
///
/// These match the DB schema defaults in `migrations/003_create_recipes.sql`
/// and are used when a recipe ingredient leaves a multiplier unset (NULL).
pub const DEFAULT_CHILD_MULTIPLIER: f64 = 0.5;
pub const DEFAULT_TEEN_MULTIPLIER: f64 = 0.75;
pub const DEFAULT_ADULT_MULTIPLIER: f64 = 1.0;

/// Attendance counts for a single meal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Attendance {
    pub children: i32,
    pub teens: i32,
    pub adults: i32,
}

impl Attendance {
    pub fn new(children: i32, teens: i32, adults: i32) -> Self {
        Self {
            children,
            teens,
            adults,
        }
    }
}

/// Number of base-recipe servings a meal's attendance represents, given the
/// recipe ingredient's per-group multipliers (an unset multiplier falls back to
/// the canonical default).
pub fn serving_multiplier(
    attendance: Attendance,
    child_multiplier: Option<f64>,
    teen_multiplier: Option<f64>,
    adult_multiplier: Option<f64>,
) -> f64 {
    let child = child_multiplier.unwrap_or(DEFAULT_CHILD_MULTIPLIER);
    let teen = teen_multiplier.unwrap_or(DEFAULT_TEEN_MULTIPLIER);
    let adult = adult_multiplier.unwrap_or(DEFAULT_ADULT_MULTIPLIER);

    attendance.children as f64 * child
        + attendance.teens as f64 * teen
        + attendance.adults as f64 * adult
}

/// Quantity of a single ingredient required for one meal:
///
/// ```text
/// base_quantity * serving_multiplier(attendance, multipliers) / base_servings
/// ```
///
/// Returns `0.0` when `base_servings <= 0`, guarding against divide-by-zero.
pub fn ingredient_quantity(
    base_quantity: f64,
    base_servings: i32,
    attendance: Attendance,
    child_multiplier: Option<f64>,
    teen_multiplier: Option<f64>,
    adult_multiplier: Option<f64>,
) -> f64 {
    if base_servings <= 0 {
        return 0.0;
    }
    base_quantity
        * serving_multiplier(
            attendance,
            child_multiplier,
            teen_multiplier,
            adult_multiplier,
        )
        / base_servings as f64
}

/// Sum attendance across distinct meals.
///
/// Ingredient rows repeat a meal's attendance once per ingredient, so callers
/// pass `(planned_meal_id, attendance)` pairs and this dedupes by meal id before
/// summing. Used for the per-day "počet jídel" (person-meals) header.
pub fn sum_distinct_meal_attendance(
    meal_attendance: impl IntoIterator<Item = (i64, Attendance)>,
) -> Attendance {
    let mut seen = std::collections::HashSet::new();
    let mut total = Attendance::new(0, 0, 0);
    for (meal_id, attendance) in meal_attendance {
        if seen.insert(meal_id) {
            total.children += attendance.children;
            total.teens += attendance.teens;
            total.adults += attendance.adults;
        }
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-9, "expected {b}, got {a}");
    }

    #[test]
    fn serving_multiplier_uses_explicit_multipliers() {
        // 2 children * 0.4 + 3 teens * 0.6 + 4 adults * 1.0 = 0.8 + 1.8 + 4.0
        let m = serving_multiplier(Attendance::new(2, 3, 4), Some(0.4), Some(0.6), Some(1.0));
        approx(m, 6.6);
    }

    #[test]
    fn serving_multiplier_falls_back_to_canonical_defaults() {
        // None => 0.5 / 0.75 / 1.0
        // 4 children * 0.5 + 4 teens * 0.75 + 2 adults * 1.0 = 2.0 + 3.0 + 2.0
        let m = serving_multiplier(Attendance::new(4, 4, 2), None, None, None);
        approx(m, 7.0);
    }

    #[test]
    fn serving_multiplier_default_is_not_one_for_children_and_teens() {
        // Regression guard against the old SQL bug that defaulted every group to
        // 1.0. A single child with an unset multiplier must weigh 0.5, not 1.0.
        let m = serving_multiplier(Attendance::new(1, 0, 0), None, None, None);
        approx(m, DEFAULT_CHILD_MULTIPLIER);
        assert_ne!(m, 1.0);
    }

    #[test]
    fn ingredient_quantity_scales_base_quantity_by_servings() {
        // base 1000 g for 10 servings; attendance represents 5 adult-servings
        // => 1000 * 5 / 10 = 500
        let q = ingredient_quantity(
            1000.0,
            10,
            Attendance::new(0, 0, 5),
            Some(0.5),
            Some(0.75),
            Some(1.0),
        );
        approx(q, 500.0);
    }

    #[test]
    fn ingredient_quantity_with_default_multipliers() {
        // 200 g base for 4 servings, 2 children + 2 adults with default mults.
        // serving_mult = 2*0.5 + 0*0.75 + 2*1.0 = 3.0
        // quantity = 200 * 3.0 / 4 = 150
        let q = ingredient_quantity(200.0, 4, Attendance::new(2, 0, 2), None, None, None);
        approx(q, 150.0);
    }

    #[test]
    fn ingredient_quantity_zero_base_servings_returns_zero() {
        let q = ingredient_quantity(500.0, 0, Attendance::new(1, 1, 1), None, None, None);
        approx(q, 0.0);
    }

    #[test]
    fn ingredient_quantity_negative_base_servings_returns_zero() {
        let q = ingredient_quantity(500.0, -3, Attendance::new(1, 1, 1), None, None, None);
        approx(q, 0.0);
    }

    #[test]
    fn ingredient_quantity_zero_attendance_is_zero() {
        let q = ingredient_quantity(999.0, 8, Attendance::new(0, 0, 0), None, None, None);
        approx(q, 0.0);
    }

    #[test]
    fn sum_distinct_meal_attendance_dedupes_repeated_meals() {
        // Meal 1 (62/44/50) appears 3 times (once per ingredient), meal 2 once.
        // Distinct sum = (62+10, 44+5, 50+6) = (72, 49, 56).
        let pairs = vec![
            (1, Attendance::new(62, 44, 50)),
            (1, Attendance::new(62, 44, 50)),
            (1, Attendance::new(62, 44, 50)),
            (2, Attendance::new(10, 5, 6)),
        ];
        let total = sum_distinct_meal_attendance(pairs);
        assert_eq!(total, Attendance::new(72, 49, 56));
    }

    #[test]
    fn sum_distinct_meal_attendance_sums_across_separate_meals() {
        // Five identical meals (62/44/50) -> 5x, matching the template's
        // arrival-day vs full-day person-meal totals.
        let pairs: Vec<(i64, Attendance)> = (1..=5)
            .map(|id| (id, Attendance::new(62, 44, 50)))
            .collect();
        let total = sum_distinct_meal_attendance(pairs);
        assert_eq!(total, Attendance::new(310, 220, 250));
    }

    #[test]
    fn sum_distinct_meal_attendance_empty_is_zero() {
        let total = sum_distinct_meal_attendance(Vec::<(i64, Attendance)>::new());
        assert_eq!(total, Attendance::new(0, 0, 0));
    }
}
