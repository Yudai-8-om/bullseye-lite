use crate::db;
use crate::models::metrics_model::Trend;
use crate::{calculate, models::earnings_model::EarningsReport};
// use chrono::NaiveDate;

/// returns the oretical net margin calculated based on the current gross margin and their industry
pub fn is_net_margin_optimized(stock_data: &EarningsReport, margin_factor: f64) -> (f64, bool) {
    let curr_gross_margin = stock_data.gross_margin.unwrap_or(100.);
    let curr_theoretical_net_margin = curr_gross_margin / margin_factor;
    let curr_net_margin = stock_data.net_margin;
    let curr_operating_margin = stock_data.operating_margin;
    let is_optimized =
        curr_theoretical_net_margin <= curr_net_margin && curr_operating_margin > curr_net_margin;
    (curr_theoretical_net_margin, is_optimized)
}

/// tells if the current net cash is at a healthy level
pub fn has_healthy_cash_position(stock_data: &EarningsReport) -> bool {
    let curr_net_income = stock_data.net_income;
    let curr_net_cash = stock_data.net_cash;
    curr_net_cash >= 0. || (-curr_net_cash / curr_net_income < 2. && curr_net_income > 0.)
}

pub fn get_short_term_trend<F>(
    target: &[EarningsReport],
    field: F,
    length: usize,
    flat_threshold: f64,
    count_threshold: usize,
) -> Trend
where
    F: Fn(&EarningsReport) -> f64,
{
    let values = db::extract_field(&target, field);
    let trend_vec = calculate::calculate_short_term_trend(&values, length, flat_threshold);
    let short_term_trend = calculate::concat_trend(trend_vec, count_threshold);
    short_term_trend
}

pub fn get_short_term_trend_option<F>(
    target: &[EarningsReport],
    field: F,
    length: usize,
    ignore_none: bool,
    flat_threshold: f64,
    count_threshold: usize,
) -> Trend
where
    F: Fn(&EarningsReport) -> Option<f64>,
{
    let values = db::extract_field(&target, field);
    let trend_vec =
        calculate::calculate_short_term_trend_option(&values, length, ignore_none, flat_threshold);
    let short_term_trend = calculate::concat_trend(trend_vec, count_threshold);
    short_term_trend
}

pub fn get_long_term_trend<F>(target: &[EarningsReport], field: F, flat_threshold: f64) -> Trend
where
    F: Fn(&EarningsReport) -> f64,
{
    let values = db::extract_field(&target, field);
    let long_term_trend = calculate::calculate_long_term_trend(&values, flat_threshold);
    long_term_trend
}

/// outputs long-term trend for the given metrics
pub fn get_long_term_trend_option<F>(
    target: &[EarningsReport],
    field: F,
    ignore_none: bool,
    flat_threshold: f64,
) -> Trend
where
    F: Fn(&EarningsReport) -> Option<f64>,
{
    let values = db::extract_field(&target, field);
    let long_term_trend =
        calculate::calculate_long_term_trend_option(&values, ignore_none, flat_threshold);
    long_term_trend
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_is_net_margin_optimized() {
//         let earnings_report = EarningsReport {
//             id: 1,
//             company_id: 1,
//             duration: "T".to_string(),
//             quarter_str: 1,
//             year_str: 2024,
//             period_ending: NaiveDate::from_ymd_opt(2024, 5, 12).unwrap(),
//             currency: "USD".to_string(),
//             net_interest_income: Some(1000.),
//             net_interest_growth_yoy: Some(1000.),
//             net_interest_margin: Some(1000.),
//             provision_for_loan_loss: Some(1000.),
//             cost_of_risk: Some(1000.),
//             revenue: 50.,
//             revenue_growth_yoy: Some(1000.),
//             cost_of_revenue: Some(1000.),
//             gross_profit: Some(1000.),
//             gross_margin: Some(1000.),
//             gross_profit_growth_yoy: Some(1000.),
//             sga_expenses: Some(1000.),
//             sga_gp_ratio: Some(1000.),
//             rnd_expenses: Some(1000.),
//             rnd_gp_ratio: Some(1000.),
//             operating_expenses: 50.,
//             operating_income: 50.,
//             operating_margin: 50.,
//             interest_expenses: Some(1000.),
//             interest_expenses_op_income_ratio: Some(1000.),
//             goodwill_impairment: 50.,
//             net_income: 50.,
//             net_margin: 50.,
//             eps_basic: 50.,
//             eps_diluted: 50.,
//             shares_outstanding_basic: 50.,
//             shares_outstanding_diluted: 50.,
//             shares_change_yoy: 50.,
//             ffo: Some(1000.),
//             ffo_margin: Some(1000.),
//             cash_and_equivalents: 50.,
//             cash_and_short_term_investments: Some(1000.),
//             total_investments: Some(1000.),
//             gross_loans: Some(1000.),
//             accounts_receivable: Some(1000.),
//             inventory: Some(1000.),
//             total_current_assets: Some(1000.),
//             goodwill: Some(1000.),
//             total_assets: 50.,
//             accounts_payable: Some(1000.),
//             total_current_liabilities: Some(1000.),
//             total_liabilities: 50.,
//             retained_earnings: 50.,
//             shareholders_equity: 50.,
//             total_debt: Some(1000.),
//             net_cash: 50.,
//             depreciation_and_amortization: Some(1000.),
//             stock_based_compensation: Some(1000.),
//             operating_cash_flow: Some(1000.),
//             operating_cash_flow_margin: Some(1000.),
//             capital_expenditure: Some(1000.),
//             investing_cash_flow: Some(1000.),
//             financing_cash_flow: Some(1000.),
//             free_cash_flow: Some(1000.),
//             free_cash_flow_margin: Some(1000.),
//             ratio_calculated: true,
//             growth_calculated: true,
//         };

//         let (theoretical_net_margin, is_optimized) = is_net_margin_optimized(&earnings_report, 3.0);
//         assert_eq!(theoretical_net_margin, 25.0);
//         assert!(is_optimized);
//     }
// }
