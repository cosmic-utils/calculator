//! Graph mode: the field's function plotted around the view and read out at a typed `x`, with
//! the engine that computes the sums and integrals qalc cannot.

mod canvas;
mod numeric;
mod sample;

pub use canvas::{Graph, Plot};

use crate::app::operations::{self, Calculator, Evaluation};
use crate::app::{self, CosmicCalculator};
use cosmic::app::Task;
use cosmic::iced::{Alignment, Length};
use cosmic::{Application, Element, widget};
use std::ops::RangeInclusive;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// What the plot, the readout and the samplings send back.
#[derive(Debug, Clone)]
pub enum Message {
    /// Points for the request of that id: all of them, or those the engine has so far.
    Plotted(u64, Result<Plot, String>),
    /// The view after a zoom or a pan: its x range and the y at its middle.
    View(RangeInclusive<f64>, f64),
    /// Samples again once the view nears the edge of the samples or changes scale, or once what
    /// was sampled stopped short.
    Resample,
    /// The x typed for the readout.
    X(String),
    /// Evaluates the function at that x.
    Evaluate,
    Evaluated(u64, Result<Evaluation, String>),
}

impl From<Message> for app::Message {
    fn from(message: Message) -> Self {
        app::Message::Graph(message)
    }
}

/// The readout under the field: the x typed, and the exact y the function takes there.
#[derive(Default)]
pub struct Readout {
    x: String,
    y: String,
    /// Incremented per readout: a superseded one stops computing.
    id: Arc<AtomicU64>,
}

/// Plotted functions in the history.
impl Calculator {
    /// A plotted function as kept in the history: it has no outcome.
    pub fn function(expression: String, decimal_comma: bool) -> Self {
        Self {
            expression,
            decimal_comma,
            ..Self::default()
        }
    }

    /// History entries of plotted functions are the ones without an outcome.
    pub fn is_function(&self) -> bool {
        self.outcome.is_empty()
    }
}

impl CosmicCalculator {
    /// The readout row: the x to evaluate at, and the y found there.
    pub(super) fn readout_row<'a>(&'a self) -> Element<'a, app::Message> {
        let spacing = cosmic::theme::active().cosmic().spacing;
        widget::row::with_capacity(3)
            .push(widget::text("x ="))
            .push(
                widget::text_input("", &self.readout.x)
                    .on_input(|x| Message::X(x).into())
                    .on_submit(|_| Message::Evaluate.into())
                    .width(Length::Fixed(160.0)),
            )
            .push(widget::text(format!("y = {}", self.readout.y)))
            .spacing(spacing.space_xs)
            .align_y(Alignment::Center)
            .into()
    }

    /// The plot, where the keypad is in the other modes.
    pub(super) fn plot_area<'a>(&'a self) -> Element<'a, app::Message> {
        widget::column::with_capacity(2)
            .push(
                widget::canvas(&self.graph)
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            // The keypad carries the toaster in the other modes.
            .push(widget::toaster(&self.toasts, widget::space::horizontal()))
            .into()
    }

    /// What graph mode does with its messages.
    pub(super) fn update_graph(&mut self, message: Message) -> Task<app::Message> {
        let mut tasks = vec![];
        match message {
            Message::Plotted(id, plot) => {
                // A native plot comes in steps: only its last one lets the next request start.
                let done = plot.as_ref().map_or(true, |plot| plot.done);
                if done && self.graph.pending == Some(id) {
                    self.graph.pending = None;
                }
                if id != self.eval_id.load(Ordering::Relaxed) {
                    return Task::batch(tasks);
                }
                match plot {
                    Ok(plot) => {
                        if plot.requested {
                            let entry = Calculator::function(
                                plot.expression.clone(),
                                self.calculator.decimal_comma,
                            );
                            // Plotting the same function again does not repeat its entry.
                            if self.config.history.last() != Some(&entry) {
                                tasks.push(self.add_to_history(entry));
                            }
                        }
                        // A new function takes the default view its samples were taken around.
                        let new = plot.expression != self.graph.expression;
                        if new {
                            self.graph.reset_view();
                        }
                        self.graph.expression = plot.expression;
                        self.graph.points = plot.points;
                        if new && !self.readout.x.trim().is_empty() {
                            tasks.push(self.update_graph(Message::Evaluate));
                        }
                        // The view may have moved on while this was sampled.
                        if done {
                            self.graph.complete = true;
                            tasks.push(self.update_graph(Message::Resample));
                        }
                    }
                    Err(error) => {
                        // Nothing more to take there; the function shown is the one a move samples.
                        self.graph.complete = true;
                        self.graph.sampling = self.graph.expression.clone();
                        tracing::error!("{}", error);
                        tasks.push(self.update(app::Message::ShowToast(error)));
                    }
                }
            }
            Message::View(range, center) => {
                self.graph.range = range;
                self.graph.y_center = Some(center);
                tasks.push(self.update_graph(Message::Resample));
            }
            Message::X(x) => self.readout.x = x,
            Message::Evaluate => {
                let (expression, x) = (self.calculator.expression.trim(), self.readout.x.trim());
                if expression.is_empty() || x.is_empty() {
                    self.readout.y.clear();
                    return Task::batch(tasks);
                }
                let id = self.readout.id.fetch_add(1, Ordering::Relaxed) + 1;
                let decimal_comma = self.calculator.decimal_comma;
                let evaluated = move |evaluation| {
                    cosmic::Action::App(Message::Evaluated(id, evaluation).into())
                };
                if let Some(formula) = numeric::Formula::shared(expression, decimal_comma) {
                    let (x, latest) = (x.to_string(), Arc::clone(&self.readout.id));
                    tasks.push(Task::perform(
                        operations::blocking(move || {
                            let superseded = || latest.load(Ordering::Relaxed) != id;
                            sample::evaluate_native(&formula, &x, decimal_comma, &superseded)
                        }),
                        evaluated,
                    ));
                    return Task::batch(tasks);
                }
                // Evaluate with full precision through the regular two-pass path.
                let calculator = Calculator {
                    expression: format!("({expression}) where x=({x})"),
                    decimal_comma,
                    ..Calculator::new()
                };
                tasks.push(Task::perform(
                    operations::blocking(move || calculator.evaluate()),
                    evaluated,
                ));
            }
            Message::Evaluated(id, evaluation) => {
                if id != self.readout.id.load(Ordering::Relaxed) {
                    return Task::batch(tasks);
                }
                match evaluation {
                    Ok(evaluation) => self.readout.y = evaluation.outcome,
                    Err(error) => {
                        tracing::error!("{}", error);
                        tasks.push(self.update(app::Message::ShowToast(error)));
                    }
                }
            }
            Message::Resample => {
                // One request at a time: its result asks again if the view moved on meanwhile.
                // What a superseded native plot found stays with the formula.
                if self.graph.needs_sampling() {
                    let expression = self.graph.expression.clone();
                    tasks.push(self.plot(expression, false));
                }
            }
        }
        Task::batch(tasks)
    }

    // Samples a function around the visible range; a newer request drops its result and kills it.
    pub(super) fn plot(&mut self, expression: String, requested: bool) -> Task<app::Message> {
        let id = self.next_request();
        self.graph.pending = Some(id);
        // A new function is sampled around the default view, the same one around its current view.
        let view = if expression == self.graph.expression {
            self.graph.range.clone()
        } else {
            Graph::DEFAULT_RANGE
        };
        let (range, samples) = self.graph.sampling(&view);
        // What the view is served by from now on: only nearing its edge asks again.
        self.graph.sampled = range.clone();
        self.graph.sampling = expression.clone();
        self.graph.complete = false;
        let calculator = self.calculator.clone();
        let latest = Arc::clone(&self.eval_id);
        let formula = numeric::Formula::shared(&expression, calculator.decimal_comma);
        self.graph.progressive = formula.is_some();
        // The engine's formulas come in runs of points, coarse to fine.
        if let Some(formula) = formula {
            let (sender, receiver) = cosmic::iced::futures::channel::mpsc::unbounded();
            std::thread::spawn(move || {
                let superseded = || latest.load(Ordering::Relaxed) != id;
                let plot = |points, done| {
                    let plot = Plot {
                        expression: expression.clone(),
                        points,
                        requested,
                        done,
                    };
                    app::Message::from(Message::Plotted(id, Ok(plot)))
                };
                let publish = |points| {
                    let _ = sender.unbounded_send(plot(points, false));
                };
                let points =
                    sample::sample_native(&formula, &range, samples, &view, &superseded, &publish);
                let _ = sender.unbounded_send(match points {
                    Ok(points) => plot(points, true),
                    Err(error) => Message::Plotted(id, Err(error)).into(),
                });
            });
            return Task::run(receiver, cosmic::Action::App);
        }
        Task::perform(
            operations::blocking(move || {
                let superseded = || latest.load(Ordering::Relaxed) != id;
                let values = calculator.sample(&expression, &range, samples, superseded)?;
                let at = |i| numeric::sample_x(&range, samples, i);
                Ok(Plot {
                    expression,
                    points: values
                        .into_iter()
                        .enumerate()
                        .map(|(i, y)| (at(i), y))
                        .collect(),
                    requested,
                    done: true,
                })
            }),
            move |plot| cosmic::Action::App(Message::Plotted(id, plot).into()),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::app::operations::Calculator;

    #[test]
    fn history_entries_tell_functions_from_calculations() {
        assert!(Calculator::function("sin(x)*x".into(), false).is_function());
        let mut calc = Calculator::new();
        calc.expression = "1+1".into();
        calc.outcome = calc.evaluate().unwrap().outcome;
        assert!(!calc.is_function());
    }
}
