use crate::calculate;
use crate::helper;
use crate::query;
use crate::schema::earnings_report;
use bullseye_api::model::BankStatement;
use bullseye_api::model::Earnings;
use bullseye_api::model::NominalStatement;
use bullseye_api::model::OtherStatement;
use bullseye_api::model::ReitsStatement;
use chrono::format::ParseError;
use chrono::NaiveDate;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Serialize)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(table_name = earnings_report)]
pub struct EarningsReport {
    pub id: i32,
    pub company_id: i32,
    pub duration: String,
    pub quarter_str: i16,
    pub year_str: i16,
    pub period_ending: NaiveDate,
    pub currency: String,
    pub net_interest_income: Option<f64>,
    pub net_interest_growth_yoy: Option<f64>,
    pub net_interest_margin: Option<f64>,
    pub provision_for_loan_loss: Option<f64>,
    pub cost_of_risk: Option<f64>,
    pub revenue: f64,
    pub revenue_growth_yoy: Option<f64>,
    pub cost_of_revenue: Option<f64>,
    pub gross_profit: Option<f64>,
    pub gross_margin: Option<f64>,
    pub gross_profit_growth_yoy: Option<f64>,
    pub sga_expenses: Option<f64>,
    pub sga_gp_ratio: Option<f64>,
    pub rnd_expenses: Option<f64>,
    pub rnd_gp_ratio: Option<f64>,
    pub operating_expenses: f64,
    pub operating_income: f64,
    pub operating_margin: f64,
    pub interest_expenses: Option<f64>,
    pub interest_expenses_op_income_ratio: Option<f64>,
    pub goodwill_impairment: f64,
    pub net_income: f64,
    pub net_margin: f64,
    pub eps_basic: f64,
    pub eps_diluted: f64,
    pub shares_outstanding_basic: f64,
    pub shares_outstanding_diluted: f64,
    pub shares_change_yoy: f64,
    pub ffo: Option<f64>,
    pub ffo_margin: Option<f64>,
    pub cash_and_equivalents: f64,
    pub cash_and_short_term_investments: Option<f64>,
    pub total_investments: Option<f64>,
    pub gross_loans: Option<f64>,
    pub accounts_receivable: Option<f64>,
    pub inventory: Option<f64>,
    pub total_current_assets: Option<f64>,
    pub goodwill: Option<f64>,
    pub total_assets: f64,
    pub accounts_payable: Option<f64>,
    pub total_current_liabilities: Option<f64>,
    pub total_liabilities: f64,
    pub retained_earnings: f64,
    pub shareholders_equity: f64,
    pub total_debt: Option<f64>,
    pub net_cash: f64,
    pub depreciation_and_amortization: Option<f64>,
    pub stock_based_compensation: Option<f64>,
    pub operating_cash_flow: Option<f64>,
    pub operating_cash_flow_margin: Option<f64>,
    pub capital_expenditure: Option<f64>,
    pub investing_cash_flow: Option<f64>,
    pub financing_cash_flow: Option<f64>,
    pub free_cash_flow: Option<f64>,
    pub free_cash_flow_margin: Option<f64>,
    pub ratio_calculated: bool,
    pub growth_calculated: bool,
}
impl EarningsReport {
    /// retrieves the lastest quarterly(TTM) earnings data for the given ticker
    pub fn latest_quarter_data(comp_id: i32, conn: &mut PgConnection) -> Result<Self, DieselError> {
        use crate::schema::earnings_report::dsl::*;
        query::load_first_row(
            earnings_report
                .filter(company_id.eq(comp_id))
                .filter(duration.eq("T"))
                .order((year_str.desc(), quarter_str.desc())),
            conn,
        )
    }
    pub fn latest_quarter_data_if_existed(
        comp_id: i32,
        conn: &mut PgConnection,
    ) -> Result<Option<Self>, DieselError> {
        use crate::schema::earnings_report::dsl::*;
        let earning = query::load_first_row(
            earnings_report
                .filter(company_id.eq(comp_id))
                .filter(duration.eq("T"))
                .order((year_str.desc(), quarter_str.desc())),
            conn,
        )
        .optional()?;
        Ok(earning)
    }

    /// retrieves the lastest annual earnings data for the given ticker
    pub fn latest_annual_data(comp_id: i32, conn: &mut PgConnection) -> Result<Self, DieselError> {
        use crate::schema::earnings_report::dsl::*;
        query::load_first_row(
            earnings_report
                .filter(company_id.eq(comp_id))
                .filter(duration.eq("Y"))
                .order(year_str.desc()),
            conn,
        )
    }

    /// retrieves the same quarter earnings data from the prvious year for the given ticker
    pub fn same_quarter_prev_year_data(
        &self,
        conn: &mut PgConnection,
    ) -> Result<Option<Self>, DieselError> {
        use crate::schema::earnings_report::dsl::*;
        let prev_year_result = query::load_first_row(
            earnings_report
                .filter(company_id.eq(&self.company_id))
                .filter(duration.eq(&self.duration))
                .filter(year_str.eq(self.year_str - 1))
                .filter(quarter_str.eq(self.quarter_str)),
            conn,
        );
        let prev_year = match prev_year_result {
            Ok(data) => Ok(Some(data)),
            Err(DieselError::NotFound) => Ok(None),
            Err(e) => Err(e),
        };
        prev_year
    }

    /// updates missing ratios and margins for the selected earnings
    pub fn update_ratios(&self, conn: &mut PgConnection) -> Result<(), DieselError> {
        use crate::schema::earnings_report::dsl::*;
        let curr_id = self.id;
        let interest_earning_assets = match (self.total_investments, self.gross_loans) {
            (Some(x), Some(y)) => Some(x + y),
            _ => None,
        };
        let nim = calculate::calculate_ratio_as_pct_option(
            self.net_interest_income,
            interest_earning_assets,
        );
        let cor = calculate::calculate_ratio_as_pct_option(
            self.provision_for_loan_loss,
            self.gross_loans,
        );
        let sga_ratio = calculate::calculate_ratio_option(self.sga_expenses, self.gross_profit);
        let rnd_ratio = calculate::calculate_ratio_option(self.rnd_expenses, self.gross_profit);
        let interest_ratio =
            calculate::calculate_ratio(self.interest_expenses, self.operating_income);
        let op_margin = (self.operating_income / self.revenue * 10000.).round() / 100.;
        let nt_margin = (self.net_income / self.revenue * 10000.).round() / 100.;
        let ocfm = calculate::calculate_ratio_as_pct(self.operating_cash_flow, self.revenue);
        let ffom = calculate::calculate_ratio_as_pct(self.ffo, self.revenue);
        query::update_earnings_table(
            curr_id,
            (
                net_interest_margin.eq(nim),
                cost_of_risk.eq(cor),
                sga_gp_ratio.eq(sga_ratio),
                rnd_gp_ratio.eq(rnd_ratio),
                interest_expenses_op_income_ratio.eq(interest_ratio),
                operating_margin.eq(op_margin),
                net_margin.eq(nt_margin),
                ffo_margin.eq(ffom),
                operating_cash_flow_margin.eq(ocfm),
                ratio_calculated.eq(true),
            ),
            conn,
        )?;
        Ok(())
    }

    /// updates missing growth rate for the selected earnings
    pub fn update_yoy_growth(&self, conn: &mut PgConnection) -> Result<(), DieselError> {
        use crate::schema::earnings_report::dsl::*;
        let curr_id = self.id;
        let prev_year_data = self.same_quarter_prev_year_data(conn)?;
        let prev_gross_profit = prev_year_data
            .as_ref()
            .map(|data| data.gross_profit)
            .flatten();
        let prev_net_interest_income = prev_year_data
            .as_ref()
            .map(|data| data.net_interest_income)
            .flatten();
        let gp_growth =
            calculate::calculate_yoy_growth_option(self.gross_profit, prev_gross_profit);
        let net_interest_income_growth = calculate::calculate_yoy_growth_option(
            self.net_interest_income,
            prev_net_interest_income,
        );
        query::update_earnings_table(
            curr_id,
            (
                net_interest_growth_yoy.eq(net_interest_income_growth),
                gross_profit_growth_yoy.eq(gp_growth),
                growth_calculated.eq(true),
            ),
            conn,
        )?;
        Ok(())
    }
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = earnings_report)]
pub struct NewEarningsReport<'a> {
    company_id: i32,
    duration: String,
    quarter_str: i16,
    year_str: i16,
    period_ending: NaiveDate,
    currency: &'a str,
    net_interest_income: Option<f64>,
    net_interest_growth_yoy: Option<f64>,
    net_interest_margin: Option<f64>,
    provision_for_loan_loss: Option<f64>,
    cost_of_risk: Option<f64>,
    revenue: f64,
    revenue_growth_yoy: Option<f64>,
    cost_of_revenue: Option<f64>,
    gross_profit: Option<f64>,
    gross_margin: Option<f64>,
    gross_profit_growth_yoy: Option<f64>,
    sga_expenses: Option<f64>,
    sga_gp_ratio: Option<f64>,
    rnd_expenses: Option<f64>,
    rnd_gp_ratio: Option<f64>,
    operating_expenses: f64,
    operating_income: f64,
    operating_margin: f64,
    interest_expenses: Option<f64>,
    interest_expenses_op_income_ratio: Option<f64>,
    goodwill_impairment: f64,
    net_income: f64,
    net_margin: f64,
    eps_basic: f64,
    eps_diluted: f64,
    shares_outstanding_basic: f64,
    shares_outstanding_diluted: f64,
    shares_change_yoy: f64,
    ffo: Option<f64>,
    ffo_margin: Option<f64>,
    cash_and_equivalents: f64,
    cash_and_short_term_investments: Option<f64>,
    total_investments: Option<f64>,
    gross_loans: Option<f64>,
    accounts_receivable: Option<f64>,
    inventory: Option<f64>,
    total_current_assets: Option<f64>,
    goodwill: Option<f64>,
    total_assets: f64,
    accounts_payable: Option<f64>,
    total_current_liabilities: Option<f64>,
    total_liabilities: f64,
    retained_earnings: f64,
    shareholders_equity: f64,
    total_debt: Option<f64>,
    net_cash: f64,
    depreciation_and_amortization: Option<f64>,
    stock_based_compensation: Option<f64>,
    operating_cash_flow: Option<f64>,
    operating_cash_flow_margin: Option<f64>,
    capital_expenditure: Option<f64>,
    investing_cash_flow: Option<f64>,
    financing_cash_flow: Option<f64>,
    free_cash_flow: Option<f64>,
    free_cash_flow_margin: Option<f64>,
    ratio_calculated: bool,
    growth_calculated: bool,
}

impl<'a> NewEarningsReport<'a> {
    /// adds new earnings data
    pub fn create_new_entry(comp_id: i32, currency: &'a str, earnings_enum: Earnings) -> Vec<Self> {
        let statement: Vec<NewEarningsReport> = match earnings_enum {
            Earnings::Nominal(val_vec) => val_vec
                .into_iter()
                .filter_map(|val| {
                    NewEarningsReport::from_nominal(comp_id, currency, val)
                        .ok()
                        .flatten()
                })
                .collect(),
            Earnings::Bank(val_vec) => val_vec
                .into_iter()
                .filter_map(|val| {
                    NewEarningsReport::from_bank(comp_id, currency, val)
                        .ok()
                        .flatten()
                })
                .collect(),
            Earnings::Reits(val_vec) => val_vec
                .into_iter()
                .filter_map(|val| {
                    NewEarningsReport::from_reits(comp_id, currency, val)
                        .ok()
                        .flatten()
                })
                .collect(),
            Earnings::Other(val_vec) => val_vec
                .into_iter()
                .filter_map(|val| {
                    NewEarningsReport::from_other(comp_id, currency, val)
                        .ok()
                        .flatten()
                })
                .collect(),
        };
        statement
    }
    fn from_nominal(
        comp_id: i32,
        currency: &'a str,
        nominal_statement: NominalStatement,
    ) -> Result<Option<Self>, ParseError> {
        if let Some((fiscal_y, fiscal_q)) =
            helper::process_fiscal_string(&nominal_statement.fiscal_quarter)
        {
            Ok(Some(NewEarningsReport {
                company_id: comp_id,
                duration: nominal_statement.term,
                quarter_str: fiscal_q,
                year_str: fiscal_y,
                period_ending: helper::convert_period_ending_str(&nominal_statement.period_ending)?,
                currency: currency,
                net_interest_income: None,
                net_interest_growth_yoy: None,
                net_interest_margin: None,
                provision_for_loan_loss: None,
                cost_of_risk: None,
                revenue: nominal_statement.revenue,
                revenue_growth_yoy: Some(nominal_statement.revenue_growth_yoy),
                cost_of_revenue: Some(nominal_statement.cost_of_revenue),
                gross_profit: Some(nominal_statement.gross_profit),
                gross_margin: Some(nominal_statement.gross_margin),
                gross_profit_growth_yoy: None,
                sga_expenses: Some(nominal_statement.sga_expenses),
                sga_gp_ratio: None,
                rnd_expenses: Some(nominal_statement.rnd_expenses),
                rnd_gp_ratio: None,
                operating_expenses: nominal_statement.operating_expenses,
                operating_income: nominal_statement.operating_income,
                operating_margin: nominal_statement.operating_margin,
                interest_expenses: Some(nominal_statement.interest_expenses),
                interest_expenses_op_income_ratio: None,
                goodwill_impairment: nominal_statement.goodwill_impairment,
                net_income: nominal_statement.net_income,
                net_margin: nominal_statement.net_margin,
                eps_basic: nominal_statement.eps_basic,
                eps_diluted: nominal_statement.eps_diluted,
                shares_outstanding_basic: nominal_statement.shares_outstanding_basic,
                shares_outstanding_diluted: nominal_statement.shares_outstanding_diluted,
                shares_change_yoy: nominal_statement.shares_change_yoy,
                ffo: None,
                ffo_margin: None,
                cash_and_equivalents: nominal_statement.cash_and_equivalents,
                cash_and_short_term_investments: Some(
                    nominal_statement.cash_and_short_term_investments,
                ),
                total_investments: None,
                gross_loans: None,
                accounts_receivable: Some(nominal_statement.accounts_receivable),
                inventory: Some(nominal_statement.inventory),
                total_current_assets: Some(nominal_statement.total_current_assets),
                goodwill: Some(nominal_statement.goodwill),
                total_assets: nominal_statement.total_assets,
                accounts_payable: Some(nominal_statement.accounts_payable),
                total_current_liabilities: Some(nominal_statement.total_current_liabilities),
                total_liabilities: nominal_statement.total_liabilities,
                retained_earnings: nominal_statement.retained_earnings,
                shareholders_equity: nominal_statement.shareholders_equity,
                total_debt: Some(nominal_statement.total_debt),
                net_cash: nominal_statement.net_cash,
                depreciation_and_amortization: Some(
                    nominal_statement.depreciation_and_amortization,
                ),
                stock_based_compensation: Some(nominal_statement.stock_based_compensation),
                operating_cash_flow: Some(nominal_statement.operating_cash_flow),
                operating_cash_flow_margin: None,
                capital_expenditure: Some(nominal_statement.capital_expenditure),
                investing_cash_flow: Some(nominal_statement.investing_cash_flow),
                financing_cash_flow: Some(nominal_statement.financing_cash_flow),
                free_cash_flow: Some(nominal_statement.free_cash_flow),
                free_cash_flow_margin: Some(nominal_statement.free_cash_flow_margin),
                ratio_calculated: false,
                growth_calculated: false,
            }))
        } else {
            Ok(None)
        }
    }
    fn from_bank(
        comp_id: i32,
        currency: &'a str,
        bank_statement: BankStatement,
    ) -> Result<Option<Self>, ParseError> {
        if let Some((fiscal_y, fiscal_q)) =
            helper::process_fiscal_string(&bank_statement.fiscal_quarter)
        {
            Ok(Some(NewEarningsReport {
                company_id: comp_id,
                duration: bank_statement.term,
                quarter_str: fiscal_q,
                year_str: fiscal_y,
                period_ending: helper::convert_period_ending_str(&bank_statement.period_ending)?,
                currency: currency,
                net_interest_income: Some(bank_statement.net_interest_income),
                net_interest_growth_yoy: None,
                net_interest_margin: None,
                provision_for_loan_loss: Some(bank_statement.provision_for_loan_loss),
                cost_of_risk: None,
                revenue: bank_statement.revenue,
                revenue_growth_yoy: Some(bank_statement.revenue_growth_yoy),
                cost_of_revenue: None,
                gross_profit: None,
                gross_margin: None,
                gross_profit_growth_yoy: None,
                sga_expenses: None,
                sga_gp_ratio: None,
                rnd_expenses: None,
                rnd_gp_ratio: None,
                operating_expenses: bank_statement.operating_expenses,
                operating_income: bank_statement.adjusted_operating_income,
                operating_margin: bank_statement.adjusted_operating_margin,
                interest_expenses: None,
                interest_expenses_op_income_ratio: None,
                goodwill_impairment: bank_statement.goodwill_impairment,
                net_income: bank_statement.net_income,
                net_margin: bank_statement.net_margin,
                eps_basic: bank_statement.eps_basic,
                eps_diluted: bank_statement.eps_diluted,
                shares_outstanding_basic: bank_statement.shares_outstanding_basic,
                shares_outstanding_diluted: bank_statement.shares_outstanding_diluted,
                shares_change_yoy: bank_statement.shares_change_yoy,
                ffo: None,
                ffo_margin: None,
                cash_and_equivalents: bank_statement.cash_and_equivalents,
                cash_and_short_term_investments: None,
                total_investments: Some(bank_statement.total_investments),
                gross_loans: Some(bank_statement.gross_loans),
                accounts_receivable: None,
                inventory: None,
                total_current_assets: None,
                goodwill: Some(bank_statement.goodwill),
                total_assets: bank_statement.total_assets,
                accounts_payable: None,
                total_current_liabilities: None,
                total_liabilities: bank_statement.total_liabilities,
                retained_earnings: bank_statement.retained_earnings,
                shareholders_equity: bank_statement.shareholders_equity,
                total_debt: Some(bank_statement.total_debt),
                net_cash: bank_statement.net_cash,
                depreciation_and_amortization: Some(bank_statement.depreciation_and_amortization),
                stock_based_compensation: Some(bank_statement.stock_based_compensation),
                operating_cash_flow: Some(bank_statement.operating_cash_flow),
                operating_cash_flow_margin: None,
                capital_expenditure: None,
                investing_cash_flow: Some(bank_statement.investing_cash_flow),
                financing_cash_flow: Some(bank_statement.financing_cash_flow),
                free_cash_flow: None,
                free_cash_flow_margin: None,
                ratio_calculated: false,
                growth_calculated: false,
            }))
        } else {
            Ok(None)
        }
    }
    fn from_reits(
        comp_id: i32,
        currency: &'a str,
        reits_statement: ReitsStatement,
    ) -> Result<Option<Self>, ParseError> {
        if let Some((fiscal_y, fiscal_q)) =
            helper::process_fiscal_string(&reits_statement.fiscal_quarter)
        {
            Ok(Some(NewEarningsReport {
                company_id: comp_id,
                duration: reits_statement.term,
                quarter_str: fiscal_q,
                year_str: fiscal_y,
                period_ending: helper::convert_period_ending_str(&reits_statement.period_ending)?,
                currency: currency,
                net_interest_income: None,
                net_interest_growth_yoy: None,
                net_interest_margin: None,
                provision_for_loan_loss: None,
                cost_of_risk: None,
                revenue: reits_statement.revenue,
                revenue_growth_yoy: Some(reits_statement.revenue_growth_yoy),
                cost_of_revenue: None,
                gross_profit: None,
                gross_margin: None,
                gross_profit_growth_yoy: None,
                sga_expenses: None,
                sga_gp_ratio: None,
                rnd_expenses: None,
                rnd_gp_ratio: None,
                operating_expenses: reits_statement.operating_expenses,
                operating_income: reits_statement.operating_income,
                operating_margin: reits_statement.operating_margin,
                interest_expenses: Some(reits_statement.interest_expenses),
                interest_expenses_op_income_ratio: None,
                goodwill_impairment: reits_statement.goodwill_impairment,
                net_income: reits_statement.net_income,
                net_margin: reits_statement.net_margin,
                eps_basic: reits_statement.eps_basic,
                eps_diluted: reits_statement.eps_diluted,
                shares_outstanding_basic: reits_statement.shares_outstanding_basic,
                shares_outstanding_diluted: reits_statement.shares_outstanding_diluted,
                shares_change_yoy: reits_statement.shares_change_yoy,
                ffo: Some(reits_statement.ffo),
                ffo_margin: None,
                cash_and_equivalents: reits_statement.cash_and_equivalents,
                cash_and_short_term_investments: None,
                total_investments: None,
                gross_loans: None,
                accounts_receivable: None,
                inventory: None,
                total_current_assets: None,
                goodwill: Some(reits_statement.goodwill),
                total_assets: reits_statement.total_assets,
                accounts_payable: None,
                total_current_liabilities: None,
                total_liabilities: reits_statement.total_liabilities,
                retained_earnings: reits_statement.retained_earnings,
                shareholders_equity: reits_statement.shareholders_equity,
                total_debt: Some(reits_statement.total_debt),
                net_cash: reits_statement.net_cash,
                depreciation_and_amortization: Some(reits_statement.depreciation_and_amortization),
                stock_based_compensation: Some(reits_statement.stock_based_compensation),
                operating_cash_flow: Some(reits_statement.operating_cash_flow),
                operating_cash_flow_margin: None,
                capital_expenditure: None,
                investing_cash_flow: None,
                financing_cash_flow: None,
                free_cash_flow: None,
                free_cash_flow_margin: None,
                ratio_calculated: false,
                growth_calculated: false,
            }))
        } else {
            Ok(None)
        }
    }
    fn from_other(
        comp_id: i32,
        currency: &'a str,
        other_statement: OtherStatement,
    ) -> Result<Option<Self>, ParseError> {
        if let Some((fiscal_y, fiscal_q)) =
            helper::process_fiscal_string(&other_statement.fiscal_quarter)
        {
            Ok(Some(NewEarningsReport {
                company_id: comp_id,
                duration: other_statement.term,
                quarter_str: fiscal_q,
                year_str: fiscal_y,
                period_ending: helper::convert_period_ending_str(&other_statement.period_ending)?,
                currency: currency,
                net_interest_income: None,
                net_interest_growth_yoy: None,
                net_interest_margin: None,
                provision_for_loan_loss: None,
                cost_of_risk: None,
                revenue: other_statement.revenue,
                revenue_growth_yoy: Some(other_statement.revenue_growth_yoy),
                cost_of_revenue: None,
                gross_profit: None,
                gross_margin: None,
                gross_profit_growth_yoy: None,
                sga_expenses: None,
                sga_gp_ratio: None,
                rnd_expenses: None,
                rnd_gp_ratio: None,
                operating_expenses: other_statement.operating_expenses,
                operating_income: other_statement.operating_income,
                operating_margin: other_statement.operating_margin,
                interest_expenses: Some(other_statement.interest_expenses),
                interest_expenses_op_income_ratio: None,
                goodwill_impairment: other_statement.goodwill_impairment,
                net_income: other_statement.net_income,
                net_margin: other_statement.net_margin,
                eps_basic: other_statement.eps_basic,
                eps_diluted: other_statement.eps_diluted,
                shares_outstanding_basic: other_statement.shares_outstanding_basic,
                shares_outstanding_diluted: other_statement.shares_outstanding_diluted,
                shares_change_yoy: other_statement.shares_change_yoy,
                ffo: None,
                ffo_margin: None,
                cash_and_equivalents: other_statement.cash_and_equivalents,
                cash_and_short_term_investments: None,
                total_investments: None,
                gross_loans: None,
                accounts_receivable: None,
                inventory: None,
                total_current_assets: None,
                goodwill: Some(other_statement.goodwill),
                total_assets: other_statement.total_assets,
                accounts_payable: None,
                total_current_liabilities: None,
                total_liabilities: other_statement.total_liabilities,
                retained_earnings: other_statement.retained_earnings,
                shareholders_equity: other_statement.shareholders_equity,
                total_debt: Some(other_statement.total_debt),
                net_cash: other_statement.net_cash,
                depreciation_and_amortization: Some(other_statement.depreciation_and_amortization),
                stock_based_compensation: Some(other_statement.stock_based_compensation),
                operating_cash_flow: Some(other_statement.operating_cash_flow),
                operating_cash_flow_margin: None,
                capital_expenditure: None,
                investing_cash_flow: Some(other_statement.investing_cash_flow),
                financing_cash_flow: Some(other_statement.financing_cash_flow),
                free_cash_flow: Some(other_statement.free_cash_flow),
                free_cash_flow_margin: Some(other_statement.free_cash_flow_margin),
                ratio_calculated: false,
                growth_calculated: false,
            }))
        } else {
            Ok(None)
        }
    }
}

/// inserts multiple earnings to the database
pub fn insert_earnings_report_batch(
    earnings_entries: Vec<NewEarningsReport>,
    conn: &mut PgConnection,
) -> Result<bool, DieselError> {
    use crate::schema::earnings_report::dsl::*;
    let update_count = diesel::insert_into(earnings_report)
        .values(&earnings_entries)
        .on_conflict((company_id, duration, quarter_str, year_str))
        .do_nothing()
        .execute(conn)?;
    Ok(update_count > 0)
}
